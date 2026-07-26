using System.Collections.ObjectModel;
using Sim.Frontend.Models;

namespace Sim.Frontend.Views;

public partial class AnalyzeView : WorkspaceView
{
    public AnalyzeView(Workspace workspace, ObservableCollection<Node> selected) : base(workspace, selected)
    {
        InitializeComponent();
    }
}