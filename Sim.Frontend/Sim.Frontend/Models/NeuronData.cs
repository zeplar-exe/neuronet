namespace Sim.Frontend.Models;

public struct NeuronData
{
    public int Id { get; set; }
    public string Model { get; set; }
    public int StateIndex { get; set; }
    public int[] Outgoing { get; set; }
}