using System;
using System.Collections.ObjectModel;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Input;
using Sim.Frontend.Models;

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

    public const double NodeRadius = 40;
    
    public double PanX { get; set; }
    public double PanY { get; set; }
    public double Zoom { get; set; } = 1;
    
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
    }

    private void OnPointerPressed(object? sender, PointerPressedEventArgs e)
    {
        var pos = e.GetPosition(this) + new Point(PanX, PanY);
        
        if ("tool" == "true")
        {
            
        }
        
        // PERF: use a quadtree
        foreach (var node in Workspace.GetAllNodes())
        {
            var nodePos = new Point(node.PositionX, node.PositionY);
            var dist = Math.Sqrt(Math.Pow(pos.X - nodePos.X, 2) + Math.Pow(pos.Y - nodePos.Y, 2));

            if (dist < NodeRadius * Zoom)
            {
                if (!e.KeyModifiers.HasFlag(KeyModifiers.Control) && !e.KeyModifiers.HasFlag(KeyModifiers.Shift))
                {
                    Selected.Clear();
                }
                
                Selected.Add(node);
            }
        }
    }

    private void OnPointerReleased(object? sender, PointerReleasedEventArgs e)
    {
        throw new System.NotImplementedException();
    }

    private void OnPointerMoved(object? sender, PointerEventArgs e)
    {
        throw new System.NotImplementedException();
    }

    private void OnPointerEntered(object? sender, PointerEventArgs e)
    {
        throw new System.NotImplementedException();
    }

    private void OnPointerExited(object? sender, PointerEventArgs e)
    {
        throw new System.NotImplementedException();
    }
}