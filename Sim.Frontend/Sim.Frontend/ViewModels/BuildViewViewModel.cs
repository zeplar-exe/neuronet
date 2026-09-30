using CommunityToolkit.Mvvm.Input;

namespace Sim.Frontend.ViewModels;

public enum BuildTool
{
    Select,
    CreateNode,
    ConnectNode,
    DeleteNode
}

public partial class BuildViewViewModel : WorkspaceViewModel
{
    public BuildTool CurrentTool { get; set; } = BuildTool.Select;

    [RelayCommand]
    public void SetTool(BuildTool tool)
    {
        CurrentTool = tool;
        OnPropertyChanged(nameof(CurrentTool));
    }
}
