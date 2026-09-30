using System;
using System.Collections.ObjectModel;
using CommunityToolkit.Mvvm.Input;
using CsBindgen;

namespace Sim.Frontend.ViewModels;

public unsafe partial class ExecuteViewViewModel : WorkspaceViewModel
{
    internal Runstate* BaseRunstate { get; set; }
    internal Runstate* CurrentRunstate { get; set; }
    public int SelectedRunstateIndex { get; set; }
    public ObservableCollection<string> RecentRunstates { get; set; } = [];
    public bool IsRunning { get; set; }

    public ExecuteViewViewModel()
    {
        BaseRunstate = (Runstate*)IntPtr.Zero;
        BaseRunstate = (Runstate*)IntPtr.Zero;
        
        RefreshRecentRunstates();
    }
    
    public void RefreshRecentRunstates()
    {
        RecentRunstates.Clear();
        
        RecentRunstates.Add("<New Runstate>");
    }

    [RelayCommand]
    public void NewRunstate()
    {
        SelectedRunstateIndex = 0;
    }

    [RelayCommand]
    public void SetRunning(bool isRunning)
    {
        IsRunning = isRunning;
    }
}