using System.Collections.ObjectModel;
using Sim.Frontend.Models;
using Sim.Frontend.ViewModels;

namespace Sim.Frontend.Views;

public partial class AnalyzeView : WorkspaceView
{
    public override WorkspaceViewModel ViewModel => new WorkspaceViewModel();
    
    public AnalyzeView(Workspace workspace, ObservableCollection<Node> selected) : base(workspace, selected)
    {
        InitializeComponent();
    }
}