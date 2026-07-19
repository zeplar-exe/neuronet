using System;
using System.Collections.ObjectModel;
using Avalonia;

namespace Sim.Frontend.Models;

public class Node : INode
{
    private static string[] GreekLetters =
    [
        "Alpha", "Beta", "Gamma", "Delta", "Epsilon", "Zeta", "Eta", "Theta", "Iota", "Kappa", "Lambda", "Mu", "Nu",
        "Xi", "Omicron", "Pi", "Rho", "Sigma", "Tau", "Upsilon", "Phi", "Chi", "Psi", "Omega",
    ];
    
    public NeuronData Neuron { get; set; }
    public string Name { get; set; }
    public int Parent { get; set; }
    public int PositionX { get; set; }
    public int PositionY { get; set; }

    public Node()
    {
        var letter = GreekLetters[Random.Shared.Next(GreekLetters.Length)];
        var hexBuffer = new byte[2];
        Random.Shared.NextBytes(hexBuffer);
        var hex = Convert.ToHexString(hexBuffer);

        Name = $"{letter} {hex}";
    }
}