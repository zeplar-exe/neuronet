using System.Collections.ObjectModel;
using System.ComponentModel.DataAnnotations;
using Avalonia;
using Avalonia.Input;
using CsBindgen;
using ReactiveUI;
using Sim.Frontend.Models;
using Sim.Frontend.ViewModels;

namespace Sim.Frontend.Views;

// fix click positions
// add Create Neuron Population
    // this one is allowed to have a lot of parameters (advanced pullout), like how to instantiate everything and positioning
// need Ctrl/Shift modifiers for edge creation
// need groups and Cmd/Ctrl+G on selected
// need functioning context menu in explorer and on-canvas
    // group (with hotkey)
    // delete (with hotkey)
    // select all (with hotkey)
// need runstate storage and retrieval
// need stimuli placement in execute view; they look like normal neurons but only have outgoing connections
// need manual ticking and play/pause execution
    // need lighting up of neurons, synapses (if possible) as current flows
// for later: either in-app sandboxed Python scripts or out-of-app scripts that push through a socket

public partial class BuildView : WorkspaceView
{
    public BuildViewViewModel BuildViewModel => (BuildViewViewModel)DataContext!;
    public override WorkspaceViewModel ViewModel => BuildViewModel;
    
    public BuildView(Workspace workspace, ObservableCollection<Node> selected) : base(workspace, selected)
    {
        InitializeComponent();

        DataContext = new BuildViewViewModel();
        ToolbarRoot.DataContext = BuildViewModel;

        BuildViewModel.PropertyChanged += (sender, args) =>
        {
            if (args.PropertyName == nameof(BuildViewModel.CurrentTool))
            {
                SelectedTool = BuildViewModel.CurrentTool switch
                {
                    BuildTool.Select => new SelectTool(Workspace, this),
                    BuildTool.CreateNode => new CreateNodeTool(Workspace, this),
                    BuildTool.DeleteNode => new DeleteNodeTool(Workspace, this),
                    _ => null
                };
            }
        };

        BuildViewModel.SetTool(BuildTool.Select);
    }

    public class SelectTool : ReactiveObject, ITool
    {
        private const double DragThreshold = 4.0; // px

        private Workspace Workspace { get; }
        private BuildView View { get; }
        private Point PressPosition { get; set; }
        private Node? HitNode { get; set; }
        private bool IsDragging { get; set; }
        private bool ShiftHeld { get; set; }

        public SelectTool(Workspace workspace, BuildView view)
        {
            Workspace = workspace;
            View = view;
        }

        public void OnPointerPressed(Point position, PointerPressedEventArgs e)
        {
            PressPosition = position;
            IsDragging = false;
            ShiftHeld = e.KeyModifiers.HasFlag(KeyModifiers.Shift);

            var nodes = Workspace.NodeKdTree.RadialSearch([position.X, position.Y], NodeRadius, 1);
            HitNode = nodes is { Length: > 0 } ? nodes[0].Value : null;
        }

        public void OnPointerReleased(Point position, PointerReleasedEventArgs e)
        {
            if (!IsDragging)
            {
                if (HitNode == null)
                {
                    View.Selected.Clear();
                }
                else if (ShiftHeld)
                {
                    if (View.Selected.Contains(HitNode))
                        View.Selected.Remove(HitNode);
                    else
                        View.Selected.Add(HitNode);
                }
                else
                {
                    View.Selected.Clear();
                    View.Selected.Add(HitNode);
                }
                View.InvalidateVisual();
            }

            HitNode = null;
            IsDragging = false;
        }

        public void OnPointerMoved(Point position, PointerEventArgs e)
        {
            if (HitNode == null) return;

            var d = position - PressPosition;
            var delta = new Vector(d.X, d.Y);

            if (!IsDragging)
            {
                if (delta.Length < DragThreshold) return;
                IsDragging = true;

                if (ShiftHeld)
                {
                    if (!View.Selected.Contains(HitNode))
                        View.Selected.Add(HitNode);
                }
                else if (!View.Selected.Contains(HitNode))
                {
                    View.Selected.Clear();
                    View.Selected.Add(HitNode);
                }
            }

            if (ShiftHeld && View.Selected.Count > 0)
            {
                var moveDelta = position - PressPosition;
                foreach (var node in View.Selected)
                    Workspace.MoveNode(node, node.PositionX + moveDelta.X, node.PositionY + moveDelta.Y);
                PressPosition = position;
            }
            else
            {
                Workspace.MoveNode(HitNode, position.X, position.Y);
            }

            View.InvalidateVisual();
        }
    }

    public class CreateNodeTool : ReactiveObject, ITool
    {
        private Workspace Workspace { get; }
        private BuildView View { get; }

        [Display(Name = "Model")]
        internal NeuronModelKind Model { get; set; }
    
        [Display(Name = "Refractory Period")]
        [Range(0, 5000)]
        public uint RefractoryPeriod { get; set; }

        public CreateNodeTool(Workspace workspace, BuildView view)
        {
            Workspace = workspace;
            View = view;
        }

        public void OnPointerPressed(Point position, PointerPressedEventArgs e)
        {
            Workspace.AddNode(Model, position.X, position.Y);
            View.InvalidateVisual();
        }

        public void OnPointerReleased(Point position, PointerReleasedEventArgs e)
        {
            
        }

        public void OnPointerMoved(Point position, PointerEventArgs e)
        {
            
        }
    }
    
    public class DeleteNodeTool : ReactiveObject, ITool
    {
        private Workspace Workspace { get; }
        private BuildView View { get; }

        public DeleteNodeTool(Workspace workspace, BuildView view)
        {
            Workspace = workspace;
            View = view;
        }

        public void OnPointerPressed(Point position, PointerPressedEventArgs e)
        {
            var nodes = Workspace.NodeKdTree.RadialSearch([position.X, position.Y], NodeRadius, 1);

            if (nodes == null || nodes.Length == 0)
            {
                View.Selected.Clear();
                return;
            }

            var node = nodes[0].Value;
            
            Workspace.RemoveNode(node);
            View.InvalidateVisual();
        }

        public void OnPointerReleased(Point position, PointerReleasedEventArgs e)
        {
            
        }

        public void OnPointerMoved(Point position, PointerEventArgs e)
        {
            
        }
    }
}

public interface ITool
{
    public void OnPointerPressed(Point position, PointerPressedEventArgs e);
    public void OnPointerReleased(Point position, PointerReleasedEventArgs e);
    public void OnPointerMoved(Point position, PointerEventArgs e);
}