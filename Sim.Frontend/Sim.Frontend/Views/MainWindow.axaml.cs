using System;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Threading;
using Sim.Frontend.ViewModels;

namespace Sim.Frontend.Views;

public partial class MainWindow : Window
{
    public MainWindowViewModel ViewModel => (MainWindowViewModel)DataContext!;
    
    public MainWindow()
    {
        InitializeComponent();

        Opened += async (sender, args) =>
        {
            if (OperatingSystem.IsMacOS())
            {
                await Task.Delay(100);

                // fixes gesture recognition on window open
                // original issue: must tab out and in again to recognize magnify gesture
                Dispatcher.UIThread.Post(Activate);
            }
        };
    }

    private void OnClosing(object? sender, WindowClosingEventArgs e)
    {
        ViewModel.Workspace.Dispose();
    }
}