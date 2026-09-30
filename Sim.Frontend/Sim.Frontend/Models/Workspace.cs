using System;
using System.Collections;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.IO;
using System.Linq;
using Avalonia.Collections;
using CommunityToolkit.Mvvm.ComponentModel;
using CsBindgen;
using KdTree;
using KdTree.Math;
using Newtonsoft.Json;
using Newtonsoft.Json.Linq;

namespace Sim.Frontend.Models;

public unsafe class Workspace : IDisposable
{
    public DirectoryInfo? SaveDirectory { get; private set; }
    public FileInfo? ManifestFile { get; private set; }
    public FileInfo? NetworkFile { get; private set; }
    public DirectoryInfo? RunstateDirectory { get; private set; }
    
    internal Network* Network { get; set; }
    public Guid Guid { get; private set; }
    public string Name { get; set; }
    public ObservableCollection<string> RunstateFiles { get; } = [];
    public ObservableCollection<ITreeNode> TreeNodes { get; } = [];
    public AvaloniaDictionary<uint, Node> NodeMap { get; } = [];
    public AvaloniaDictionary<uint, Edge> EdgeMap { get; } = [];
    public KdTree<double, Node> NodeKdTree { get; set; }

    private Workspace()
    {
        NodeKdTree = new KdTree<double, Node>(dimensions: 2, new DoubleMath());
    }
    
    public static Workspace Create(string name)
    {
        var workspace = new Workspace
        {
            Network = NativeMethods.create_network(),
            Name = name,
            Guid = Guid.NewGuid(),
        };

        return workspace;
    }
    
    public static Workspace Load(string folderPath)
    {
        return Create(folderPath);
    }

    internal Node AddNode(NeuronModelKind model, double positionX, double positionY)
    {
        uint id;
        
        switch (model)
        {
            case NeuronModelKind.IntegrateFire:
                id = NativeMethods.add_integrate_fire_neuron(Network);
                break;
            case NeuronModelKind.LIF:
                id = NativeMethods.add_lif_neuron(Network);
                break;
            case NeuronModelKind.Izhikevich:
                id = NativeMethods.add_izhikevich_neuron(Network);
                break;
            default:
                throw new ArgumentOutOfRangeException(nameof(model), model, null);
        }
        
        var node = new Node { Id = id, PositionX =  positionX, PositionY = positionY };

        TreeNodes.Add(node);
        NodeMap.Add(id, node);
        NodeKdTree.Add([positionX, positionY], node);
        
        return node;
    }

    internal Edge AddEdge(Node from, Node to)
    {
        var id = NativeMethods.add_synapse(Network, from.Id, to.Id);
        var edge = new Edge { Id = id, Source = from, Target = to };

        EdgeMap.Add(id, edge);
        
        return edge;
    }

    public void MoveNode(Node node, double positionX, double positionY)
    {
        NodeKdTree.RemoveAt([node.PositionX, node.PositionY]);
        
        node.PositionX = positionX;
        node.PositionY = positionY;
        NodeKdTree.Add([node.PositionX, node.PositionY], node);
    }

    public void RemoveNode(Node node)
    {
        TreeNodes.Remove(node);
        NodeMap.Remove(node.Id);
        NodeKdTree.RemoveAt([node.PositionX, node.PositionY]);
        NativeMethods.network_remove_neuron(Network, node.Id);

        foreach (var edge in EdgeMap.Values)
        {
            if (edge.Source == node || edge.Target == node)
                RemoveEdge(edge);
        }
    }

    public void RemoveEdge(Edge edge)
    {
        EdgeMap.Remove(edge.Id);
        NativeMethods.network_remove_synapse(Network, edge.Id);
    }

    public void SaveTo(string folderPath)
    {
        var fullPath = new DirectoryInfo(Path.Join(folderPath, Guid.ToString()));

        if (fullPath.Exists)
            fullPath.Create();
        
        var manifest = new FileInfo(Path.Join(folderPath, "manifest.json"));
        using var manifestWriter = manifest.CreateText();
        var manifestJson = Serialize();
        manifestWriter.Write(JsonConvert.SerializeObject(manifestJson));
    }

    public JObject Serialize()
    {
        var settings = new JsonSerializerSettings {
            PreserveReferencesHandling = PreserveReferencesHandling.Objects
        };

        return new JObject
        {
            ["id"] = Guid.ToString(),
            ["name"] = Name,
            ["network_hash"] = 0,
            // ["nodes"] = Nodes.ToArray(),
            // ["edges"] = Edges.ToArray()
        };
    }

    public IEnumerable<Node> GetAllNodes()
    {
        IEnumerable<Node> GetNested(IEnumerable<ITreeNode> nodes)
        {
            foreach (var node in nodes)
            {
                if (node is Node n)
                {
                    yield return n;
                }
                else if (node is Group nestedGroup)
                {
                    foreach (var nested in GetNested(nestedGroup.Children))
                    {
                        yield return nested;
                    }
                }
            }
        }
        
        foreach (var node in GetNested(TreeNodes))
        {
            yield return node;
        }
    }

    public void Dispose()
    {
        GC.SuppressFinalize(this);
        
        NativeMethods.destroy_network(Network);
    }
}