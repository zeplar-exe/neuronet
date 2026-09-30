using System;
using System.Collections.ObjectModel;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Input.GestureRecognizers;
using Avalonia.Media;
using Sim.Frontend.Models;
using Sim.Frontend.ViewModels;

namespace Sim.Frontend.Views;

public abstract partial class WorkspaceView : UserControl
{
    public static readonly StyledProperty<Control> PropertiesPanelProperty =
        AvaloniaProperty.Register<WorkspaceView, Control>("PropertiesPanel");
    
    public Control PropertiesPanel
    {
        get => GetValue(PropertiesPanelProperty);
        set => SetValue(PropertiesPanelProperty, value);
    }
    
    public static readonly StyledProperty<Control> ToolbarPanelProperty =
        AvaloniaProperty.Register<WorkspaceView, Control>("ToolbarPanel");
    
    public Control ToolbarPanel
    {
        get => GetValue(ToolbarPanelProperty);
        set => SetValue(ToolbarPanelProperty, value);
    }
    
    public static readonly StyledProperty<ContextMenu> ExplorerContextMenuProperty =
        AvaloniaProperty.Register<WorkspaceView, ContextMenu>("ExplorerContextMenu");
    
    public ContextMenu ExplorerContextMenu
    {
        get => GetValue(ExplorerContextMenuProperty);
        set => SetValue(ExplorerContextMenuProperty, value);
    }

    public const double NodeRadius = 20;
    
    public abstract WorkspaceViewModel ViewModel { get; }
    
    public ITool? SelectedTool { get; set; }
    
    public Workspace Workspace { get; }
    public ObservableCollection<Node> Selected { get; }

    public WorkspaceView(Workspace workspace, ObservableCollection<Node> selected)
    {
        Workspace = workspace;
        Selected = selected;

        PointerPressed += OnPointerPressed;
        PointerReleased += OnPointerReleased;
        PointerMoved += OnPointerMoved;
        PointerEntered += OnPointerEntered;
        PointerExited += OnPointerExited;
        
        Background = Brushes.Transparent;
        
        GestureRecognizers.Add(new PinchGestureRecognizer());
        GestureRecognizers.Add(new ScrollGestureRecognizer());
        GestureRecognizers.Add(new PullGestureRecognizer());
        
        AddHandler(Gestures.PointerTouchPadGestureMagnifyEvent, (sender, args) =>
        {
            if (args.Delta.Y > 0)
            {
                ViewModel.ZoomIn();   
            }
            else if (args.Delta.Y < 0)
            {
                ViewModel.ZoomOut();
            }
            
            InvalidateVisual();
        });
        PointerWheelChanged += (sender, args) =>
        {
            var d = args.Delta;
            ViewModel.Pan(d.X, d.Y);
            
            InvalidateVisual();
        };
    }

    public override void Render(DrawingContext context)
    {
        var center = new Point(ViewModel.PanX, ViewModel.PanY);
        var centerOffset = center + new Vector(Bounds.Width / 2d, Bounds.Height / 2d);
        
        var nodePen = new Pen(Brushes.Black, 2);
        var edgePen = new Pen(Brushes.Green, 2);
        
        foreach (var node in Workspace.GetAllNodes())
        {
            var brush = Brushes.CadetBlue;
            if (Selected.Contains(node))
                brush = Brushes.DarkBlue;
            context.DrawEllipse(brush, nodePen, new Point(node.PositionX, node.PositionY) + centerOffset, NodeRadius, NodeRadius);
        }

        foreach (var edge in Workspace.EdgeMap.Values)
        {
            var p1 = new Point(edge.Source.PositionX, edge.Source.PositionY) + centerOffset;
            var p2 = new Point(edge.Target.PositionX, edge.Target.PositionY) + centerOffset;
            var dir = new Vector(p2.X - p1.X, p2.Y - p1.Y).Normalize();
            p1 += dir * 5;
            p2 -= dir * (NodeRadius + edgePen.Thickness);
            context.DrawLine(edgePen, p1, p2);
            
            var rot = new Vector(-dir.Y, dir.X);
            context.DrawLine(edgePen,
                p2 + rot * (NodeRadius / 2),
                p2 + rot * -(NodeRadius / 2));
        }
    }

    private void OnPointerPressed(object? sender, PointerPressedEventArgs e)
    {
        var pos = e.GetPosition(this) + new Point(ViewModel.PanX, ViewModel.PanY);

        if (SelectedTool != null)
        {
            SelectedTool.OnPointerPressed(pos, e);
            
            return;
        }
        
        foreach (var kdn in Workspace.NodeKdTree.RadialSearch([pos.X, pos.Y], NodeRadius, 999))
        {
            var node = kdn.Value;
            var dist = new Vector(node.PositionX, node.PositionY).Length;

            if (dist < NodeRadius * ViewModel.Zoom)
            {
                if (!e.KeyModifiers.HasFlag(KeyModifiers.Control) && !e.KeyModifiers.HasFlag(KeyModifiers.Shift))
                {
                    Selected.Clear();
                }
                
                Selected.Add(node);
            }
        }
    }

    protected virtual void OnPointerReleased(object? sender, PointerReleasedEventArgs e)
    {
        var pos = e.GetPosition(this) + new Point(ViewModel.PanX, ViewModel.PanY);

        SelectedTool?.OnPointerReleased(pos, e);
    }

    protected virtual void OnPointerMoved(object? sender, PointerEventArgs e)
    {
        var pos = e.GetPosition(this) + new Point(ViewModel.PanX, ViewModel.PanY);

        SelectedTool?.OnPointerMoved(pos, e);
    }

    protected virtual void OnPointerEntered(object? sender, PointerEventArgs e)
    {
        
    }

    protected virtual void OnPointerExited(object? sender, PointerEventArgs e)
    {
        
    }
}