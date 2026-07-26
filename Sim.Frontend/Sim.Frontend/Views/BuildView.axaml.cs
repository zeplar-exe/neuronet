using System;
using System.Collections.ObjectModel;
using System.ComponentModel.DataAnnotations;
using System.Linq;
using System.Reflection;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Markup.Xaml;
using Avalonia.Media;
using Avalonia.Remote.Protocol.Input;
using CsBindgen;
using ReactiveUI;
using Sim.Frontend.Models;

namespace Sim.Frontend.Views;

public partial class BuildView : WorkspaceView
{
    public bool CreateNodeToolSelected { get; set; }
    public bool ConnectNodeToolSelected { get; set; }
    public bool DeleteNodeToolSelected { get; set; }
    
    public int LevelOfDetail { get; set; }
    
    public BuildView(Workspace workspace, ObservableCollection<Node> selected) : base(workspace, selected)
    {
        InitializeComponent();
    }

    protected override void OnMeasureInvalidated()
    {
        base.OnMeasureInvalidated();
        
        // there can be items in the explorer that aren't visible in the workspace
            // items in the explorer that aren't visible in the workspace are grabbed, can be panned to
        // in general, on panning/zooming (handled in base class), have to invalidate *slowly*
        
    }

    public override void Render(DrawingContext context)
    {
        base.Render(context);
        
        var nodePen = new Pen(Brushes.Black, 2);
        var edgePen = new Pen(Brushes.Green, 2);
        
        foreach (var node in Workspace.GetAllNodes())
        {
            context.DrawEllipse(Brushes.CadetBlue, nodePen, new Point(node.PositionX, node.PositionY), NodeRadius, NodeRadius);
        }

        if (LevelOfDetail < 2)
        {
            foreach (var edge in Workspace.Edges)
            {
                context.DrawLine(edgePen,
                    new Point(edge.Source.PositionX, edge.Source.PositionY),
                    new Point(edge.Target.PositionX, edge.Target.PositionY));
                var vector = new Vector(edge.Target.PositionX - edge.Source.PositionX,
                    edge.Target.PositionY - edge.Source.PositionY);
                context.DrawLine(edgePen,
                    new Point(edge.Target.PositionX, edge.Target.PositionY),
                    new Point(edge.Target.PositionX + vector.X / 2, edge.Target.PositionY + vector.Y / 2));
            }
        }
    }

    public class CreateNodeTool : ReactiveObject, ITool
    {
        private Workspace Workspace { get; }
        
        [Display(Name = "Model")]
        internal NeuronModelKind Model { get; set; }
    
        [Display(Name = "Refractory Period")]
        [Range(0, 10000)]
        public uint RefractoryPeriod { get; set; }

        public CreateNodeTool(Workspace workspace)
        {
            Workspace = workspace;
        }

        public void OnPointerPressed(Point position, PointerPressedEventArgs e)
        {
            uint id;

            unsafe
            {
                switch (Model)
                {
                    case NeuronModelKind.IntegrateFire:
                        id = NativeMethods.add_integrate_fire_neuron(Workspace.Network, RefractoryPeriod);
                        break;
                    case NeuronModelKind.LIF:
                        id = NativeMethods.add_lif_neuron(Workspace.Network, RefractoryPeriod);
                        break;
                    case NeuronModelKind.Izhikevich:
                        id = NativeMethods.add_izhikevich_neuron(Workspace.Network, RefractoryPeriod);
                        break;
                    default:
                        throw new ArgumentOutOfRangeException();
                }
            }

            var node = new Node
            {
                Id = id,
                Model = "model",
                PositionX = position.X,
                PositionY = position.Y
            };
            
            Workspace.Nodes.Add(node);
        }

        public void OnPointerReleased(Point position, PointerReleasedEventArgs e)
        {
            
        }

        public void OnPointerMoved(Point position, PointerEventArgs e)
        {
            
        }
    }
}

public interface ITool
{
    public void OnPointerPressed(Point position, PointerPressedEventArgs e);
    public void OnPointerReleased(Point position, PointerReleasedEventArgs e);
    public void OnPointerMoved(Point position, PointerEventArgs e);
}