using System.Collections.ObjectModel;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Markup.Xaml;
using Avalonia.Media;
using Sim.Frontend.Models;

namespace Sim.Frontend.Views;

public partial class BuildView : WorkspaceView
{
    public bool CreateNodeToolSelected { get; set; }
    public bool ConnectNodeToolSelected { get; set; }
    public bool DeleteNodeToolSelected { get; set; }
    
    public int LevelOfDetail { get; set; }
    
    public BuildView(ObservableCollection<Node> nodes, ObservableCollection<Edge> edges, ObservableCollection<Node> selected) : base(nodes, edges, selected)
    {
        InitializeComponent();
    }

    protected override void OnMeasureInvalidated()
    {
        base.OnMeasureInvalidated();
    }

    public override void Render(DrawingContext context)
    {
        base.Render(context);
        
        var nodePen = new Pen(Brushes.Black, 2);
        var edgePen = new Pen(Brushes.Green, 2);
        
        foreach (var node in Nodes)
        {
            context.DrawEllipse(Brushes.CadetBlue, nodePen, new Point(node.PositionX, node.PositionY), 5, 5);
        }

        if (LevelOfDetail < 2)
        {
            foreach (var edge in Edges)
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
}