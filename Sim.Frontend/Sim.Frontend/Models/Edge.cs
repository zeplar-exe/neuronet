namespace Sim.Frontend.Models;

public class Edge
{
    public SynapseData Synapse { get; set; }
    public Node Source { get; set; }
    public Node Target { get; set; }

    public Edge(Node source, Node target)
    {
        Source = source;
        Target = target;
    }
}