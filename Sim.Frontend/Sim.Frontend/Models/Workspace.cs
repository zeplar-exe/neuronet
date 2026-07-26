using System;
using System.Collections;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.IO;
using System.Linq;
using CommunityToolkit.Mvvm.ComponentModel;
using CsBindgen;
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
    public string Name { get; private set; }
    public ObservableCollection<string> RunstateFiles { get; } = [];
    public ObservableCollection<INode> Nodes { get; } = [];
    public ObservableCollection<Edge> Edges { get; } = [];
    
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
        
    }

    public void SaveTo(string folderPath)
    {
        var fullPath = new DirectoryInfo(Path.Join(folderPath, Guid.ToString()));

        if (fullPath.Exists)
            throw;
        
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
            ["nodes"] = Nodes.ToArray(),
            ["edges"] = Edges.ToArray()
        };
    }

    public IEnumerable<Node> GetAllNodes()
    {
        IEnumerable<Node> GetNested(IEnumerable<INode> nodes)
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
        
        foreach (var node in GetNested(Nodes))
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