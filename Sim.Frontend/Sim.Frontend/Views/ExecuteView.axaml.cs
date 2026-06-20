using System.Collections.ObjectModel;
using Sim.Frontend.Models;

namespace Sim.Frontend.Views;

public partial class ExecuteView : WorkspaceView
{
    public ExecuteView(ObservableCollection<Node> nodes, ObservableCollection<Edge> edges, ObservableCollection<Node> selected) : base(nodes, edges, selected)
    {
        InitializeComponent();
    }
}