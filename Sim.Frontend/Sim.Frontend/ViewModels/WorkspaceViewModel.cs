using System;

namespace Sim.Frontend.ViewModels;

public class WorkspaceViewModel : ViewModelBase
{
    private int ZoomLevel { get; set; } = 0;
    private double ZoomBase => 1.05d;
    
    public double PanX { get; set; }
    public double PanY { get; set; }
    public double Zoom { get; set; } = 1;
    
    public void Pan(double x, double y)
    {
        PanX += x;
        PanY += y;
    }

    public void ZoomIn()
    {
        ZoomLevel += 1;
        Zoom = Math.Pow(ZoomBase, ZoomLevel);
    }

    public void ZoomOut()
    {
        ZoomLevel -= 1;
        Zoom = Math.Pow(ZoomBase, ZoomLevel);
    }
}