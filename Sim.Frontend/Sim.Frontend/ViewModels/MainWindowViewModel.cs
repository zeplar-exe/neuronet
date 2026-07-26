using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.ComponentModel;
using Avalonia;
using Avalonia.Controls.ApplicationLifetimes;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Sim.Frontend.Models;
using Sim.Frontend.Views;

namespace Sim.Frontend.ViewModels;

public partial class MainWindowViewModel : ViewModelBase
{
    public BuildView BuildView { get; }
    public ExecuteView ExecuteView { get; }
    public AnalyzeView AnalyzeView { get; }

    public ObservableCollection<string> ViewTabSource =>
    [
        "Build",
        "Execute",
        "Analyze"
    ];
    
    [ObservableProperty]
    public partial string SelectedViewTab { get; set; } = "Build";

    public WorkspaceView CurrentView
    {
        get;
        set
        {
            field = value;
            OnPropertyChanged();
            OnPropertyChanged(nameof(BuildViewOpen));
            OnPropertyChanged(nameof(ExecuteViewOpen));
            OnPropertyChanged(nameof(AnalyzeViewOpen));
        }
    }

    public Workspace Workspace { get; }
    public ObservableCollection<INode> Nodes { get; } = [];
    public ObservableCollection<Edge> Edges { get; } = [];
    public ObservableCollection<Node> Selected { get; } = [];
    
    public bool BuildViewOpen => CurrentView == BuildView;
    public bool ExecuteViewOpen => CurrentView == ExecuteView;
    public bool AnalyzeViewOpen => CurrentView == AnalyzeView;

    public MainWindowViewModel()
    {
        Workspace = new Workspace("Workspace", "./");
        Nodes = Workspace.Nodes;
        Edges = Workspace.Edges;
        Nodes.Add(new Node());
        Nodes.Add(new Node());
        Nodes.Add(new Node());
        Nodes.Add(new Node());
        
        BuildView = new BuildView(Workspace, Selected);
        ExecuteView = new ExecuteView(Workspace, Selected);
        AnalyzeView = new AnalyzeView(Workspace, Selected);
        CurrentView = BuildView;
    }
    
    [RelayCommand]
    public void OpenSettings()
    {
        if (Application.Current?.ApplicationLifetime is IClassicDesktopStyleApplicationLifetime { MainWindow: { } main})
        {
            var window = new SettingsWindow();
            
            window.ShowDialog(main);
        }
    }

    [RelayCommand]
    public void Exit()
    {
        if (Application.Current?.ApplicationLifetime is IClassicDesktopStyleApplicationLifetime desktop)
        {
            desktop.Shutdown();
        }
    }

    protected override void OnPropertyChanged(PropertyChangedEventArgs e)
    {
        base.OnPropertyChanged(e);
        
        if (e.PropertyName == nameof(SelectedViewTab))
        {
            switch (SelectedViewTab)
            {
                case "Build":
                    CurrentView = BuildView;
                    break;
                case "Execute":
                    CurrentView = ExecuteView;
                    break;
                case "Analyze":
                    CurrentView = AnalyzeView;
                    break;
            }
        }
    }
}