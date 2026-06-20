using System.Collections.ObjectModel;
using Avalonia;
using Avalonia.Controls;
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
    
    public ObservableCollection<Node> Nodes { get; } = [];
    public ObservableCollection<Edge> Edges { get; } = [];
    public ObservableCollection<Node> Selected { get; } = [];

    public WorkspaceView(ObservableCollection<Node> nodes, ObservableCollection<Edge> edges, ObservableCollection<Node> selected)
    {
        Nodes = nodes;
        Edges = edges;
        Selected = selected;
    }
}