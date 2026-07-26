using System;

namespace Sim.Frontend.Models;

public class AppSettings
{
    // recall the 5001 limit
    [SettingPath("build/synapse/default_conduction_time")]
    public int DefaultSynapseConductionTime { get; set; } = 0;
    [SettingPath("build/synapse/default_conduction_velocity")]
    public int DefaultSynapseConductionVelocity { get; set; } = int.MaxValue;

    [SettingPath("execute/runstate/recent_limit")]
    public int RecentRunstateLimit { get; set; } = 100;

    [AttributeUsage(AttributeTargets.Property)]
    public class SettingPathAttribute : Attribute
    {
        public string Path { get; set; }

        public SettingPathAttribute(string path)
        {
            Path = path;
        }
    }
}