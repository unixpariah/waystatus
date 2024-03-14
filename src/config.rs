use crate::{
    util::{BacklightOpts, BatteryOpts, RamOpts, Trigger},
    Cmd,
};
use smithay_client_toolkit::shell::wlr_layer::Anchor;

/*               _                        __ _                       _   _
 *       ___ ___| |__     ___ ___  _ __  / _(_) __ _ _   _ _ __ __ _| |_(_) ___  _ __
 *      / __/ __| '_ \   / __/ _ \| '_ \| |_| |/ _` | | | | '__/ _` | __| |/ _ \| '_ \
 *      \__ \__ \ |_) | | (_| (_) | | | |  _| | (_| | |_| | | | (_| | |_| | (_) | | | |
 *      |___/___/_.__/   \___\___/|_| |_|_| |_|\__, |\__,_|_|  \__,_|\__|_|\___/|_| |_|
 *                                             |___/
 */

// Value to display when data is not available
pub const UNKOWN: &str = "N/A";

// Background color              R   G   B   A
pub const BACKGROUND: [u8; 4] = [20, 15, 33, 255];

/*
 * Placement of status bar
 * Options:
 * - Anchor::TOP
 * - Anchor::BOTTOM
 */
pub const PLACEMENT: Anchor = Anchor::TOP;
pub const HEIGHT: i32 = 40; // Height of status bar in pixels
pub const FONT: Font = Font {
    family: "JetBrainsMono Nerd Font",
    size: 16.0,
    bold: true,
    //      R    G    B
    color: [255, 255, 255],
};

/*
 *  Function          Description                                                Argument                                                                 Example
 *  --------------    ---------------------------------------------------------  -----------------------------------------------------------------------  ----------------
 *  Cmd::Custom       This function allows you to run any custom command         It takes two arguments: the command to run and its arguments             ("pamixer", "--get-volume")
 *                    that you specify.                                          These are typically passed as strings.
 *
 *  Cmd::Workspaces   This function is used to display the status of all         It takes two arguments: icons for active and inactive windows            (" ", " ")
 *                    workspaces.                                                These are typically passed as strings.
 *
 *  Cmd::Backlight    This function provides information about the backlight     It takes one argument: an enum representing the type of data to display  (BacklightOpts::Perc)
 *                    of your display.                                           This could be the percentage of brightness, the actual brightness value,
 *                                                                               or any other relevant information.
 *
 *  Cmd::Ram          This function gives information about the system's RAM.    It takes one argument: an enum representing the type of data to display   (RamOpts::PercUsed)
 *                                                                               This could be the percentage of RAM used, the actual amount of RAM used,
 *                                                                               or any other relevant information.
 *
 *  Cmd::Cpu          This function provides information about the CPU.          It takes one argument: an enum representing the type of data to display   (CpuOpts::Perc)
 *                                                                               This could be the percentage of CPU used, the actual CPU speed,
 *                                                                               or any other relevant information.
 *
 *  Cmd::Battery      This function provides information about the battery.      It takes one argument: an enum representing the type of data to display   (BatteryOpts::Capacity)
 *
 *
 *  The COMMAND_CONFIGS array is a static array of tuples. Each tuple represents a command to be executed, along with its associated properties.
 *
 *  Tuple Element    Description
 *  --------------   ---------------------------------------------------------------------------------------------------
 *  Command          The command to be executed. This could be a custom command or one of the predefined commands.
 *
 *  x                The x-coordinate where the output of the command will be displayed on the screen.
 *
 *  y                The y-coordinate where the output of the command will be displayed on the screen.
 *
 *  format           The format in which the output of the command will be displayed. The 's%' is a placeholder where the output of the command will be placed.
 *
 *  Trigger          The event that will trigger the command to be executed. This could be a time interval, a workspace change, or any other relevant event.
 *
 */

const BACKLIGHT_PATH: &str = "/sys/class/backlight/intel_backlight/brightness";
const BATTERY_PATH: &str = "/sys/class/power_supply/BAT0/capacity";

// The number of command entries in the COMMAND_CONFIGS array. We use a static array for more efficient access.
const COMMAND_NUM: usize = 7;

#[rustfmt::skip]
pub const COMMAND_CONFIGS: [(Cmd, f64, f64, &str, Trigger); COMMAND_NUM] = [
    // Command                                x        y      format    Trigger
    (Cmd::Custom("pamixer", "--get-volume"),  1540.0,  25.0,  " s%%",  Trigger::TimePassed(100)    ),
    (Cmd::Custom("date", "+%H:%M"),           925.0,   25.0,  " s%",   Trigger::TimePassed(60000)  ),
    (Cmd::Custom("iwgetid", "-r"),            1775.0,  25.0,  "  s%",  Trigger::TimePassed(60000)  ),
    (Cmd::Workspaces(" ", " "),             35.0,    25.0,  "s%",     Trigger::WorkspaceChanged   ),
    (Cmd::Backlight(BacklightOpts::Perc),     1475.0,  25.0,  "󰖨 s%%",  Trigger::FileChange(BACKLIGHT_PATH)   ),
    (Cmd::Ram(RamOpts::PercUsed),             1635.0,  25.0,  "󰍛 s%%",  Trigger::TimePassed(5000)   ),
    (Cmd::Cpu,                                1700.0,  25.0,  " s%%",  Trigger::TimePassed(5000)   ),
    //(Cmd::Battery(BatteryOpts::Capacity),     1390.0,  25.0,  " s%%",  Trigger::FileChange(BATTERY_PATH)  ),
];

pub struct Font {
    pub family: &'static str,
    pub size: f64,
    pub bold: bool,
    pub color: [u8; 3],
}
