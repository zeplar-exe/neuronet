using System;
using System.Collections.ObjectModel;
using Avalonia;
using CsBindgen;

namespace Sim.Frontend.Models;

public class Node : ITreeNode
{
    private static string[] GreekLetters =
    [
        "Alpha", "Beta", "Gamma", "Delta", "Epsilon", "Zeta", "Eta", "Theta", "Iota", "Kappa", "Lambda", "Mu", "Nu",
        "Xi", "Omicron", "Pi", "Rho", "Sigma", "Tau", "Upsilon", "Phi", "Chi", "Psi", "Omega",
    ];
    
    public uint Id { get; set; }
    internal NeuronModelKind Model { get; set; }
    public string Name { get; set; }
    public Group? Parent { get; set; }
    public double PositionX { get; set; }
    public double PositionY { get; set; }

    public Node()
    {
        var letter = GreekLetters[Random.Shared.Next(GreekLetters.Length)];
        var hexBuffer = new byte[2];
        Random.Shared.NextBytes(hexBuffer);
        var hex = Convert.ToHexString(hexBuffer);

        Name = $"{letter} {hex}";
    }
}