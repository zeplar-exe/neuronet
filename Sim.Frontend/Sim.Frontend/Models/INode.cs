namespace Sim.Frontend.Models;

public interface INode
{
    public string Name { get; }
    public Group? Parent { get; }
}