using System.Collections.ObjectModel;
using Avalonia.Media;
using Sim.Frontend.Models;
using Sim.Frontend.ViewModels;

namespace Sim.Frontend.Views;

public partial class ExecuteView : WorkspaceView
{
    public ExecuteViewViewModel ExecuteViewModel => (ExecuteViewViewModel)DataContext!;
    public override WorkspaceViewModel ViewModel => ExecuteViewModel;
    
    public ExecuteView(Workspace workspace, ObservableCollection<Node> selected) : base(workspace, selected)
    {
        InitializeComponent();

        DataContext = new ExecuteViewViewModel();
    }

    public override void Render(DrawingContext context)
    {
        base.Render(context);
    }
}