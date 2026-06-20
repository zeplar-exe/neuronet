using System.Collections.ObjectModel;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Markup.Xaml;
using Sim.Frontend.Models;

namespace Sim.Frontend.Views;

public partial class AnalyzeView : WorkspaceView
{
    public AnalyzeView(ObservableCollection<Node> nodes, ObservableCollection<Edge> edges, ObservableCollection<Node> selected) : base(nodes, edges, selected)
    {
        InitializeComponent();
    }
}