namespace Sim.Frontend.Models;

public interface ITreeNode
{
    public string Name { get; }
    public Group? Parent { get; }
}