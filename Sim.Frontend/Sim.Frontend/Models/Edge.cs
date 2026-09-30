namespace Sim.Frontend.Models;

public class Edge
{
    public uint Id { get; set; }
    public Node Source { get; set; }
    public Node Target { get; set; }
}