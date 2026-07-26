using System.Collections.ObjectModel;
using Avalonia.Interactivity;
using Avalonia.Media;
using CsBindgen;
using Sim.Frontend.Models;
using Sim.Frontend.ViewModels;

namespace Sim.Frontend.Views;

public partial class ExecuteView : WorkspaceView
{
    public ExecuteViewViewModel ViewModel => (ExecuteViewViewModel)DataContext!;
    
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