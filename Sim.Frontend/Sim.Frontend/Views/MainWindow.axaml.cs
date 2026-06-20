using Avalonia;
using Avalonia.Controls;
using Sim.Frontend.ViewModels;

namespace Sim.Frontend.Views;

public partial class MainWindow : Window
{
    public MainWindowViewModel ViewModel => (MainWindowViewModel)DataContext!;
    
    public MainWindow()
    {
        InitializeComponent();
    }
}