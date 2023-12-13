# Kodiki

### 3D Vim-like text editor with an IDE ambition built on top of Bevy, Helix Editor and WezTerm.
As of version 0.1 it is possible to write your code with it, build it and commit to git with built-in terminal emulator.

![kodiki_header](https://github.com/gavlig/kodiki/blob/kodiki_0.1/assets/readme/header.gif)  
[Timelapse Demo Video](https://rumble.com/v40lnba-kodiki-timelapse-demo.html)

### How to build
1. Make sure you have Rust installed. See https://www.rust-lang.org/
2. git lfs install (this has to be done once per OS user account, so it's optional for those who have already used lfs before)
3. git clone https://github.com/gavlig/kodiki.git
5. In prepare_runtime.sh uncomment PLATFORM, OSNAME and ARCHIVEFORMAT according to your OS.
6. Run prepare_runtime.sh to obtain runtime of a Helix Editor that this release is based on.
7. Run make_build.sh to get a working build of Kodiki in build/bin with all assets put in place.  
   Executable build/bin/kodiki should work out of the box.
8. Alternatively use run.sh to invoke cargo run -r with HELIX_RUNTIME pointing towards build/bin/runtime which is required for Helix to work
(these scripts were used on Linux, Mac and Windows([git bash](https://gitforwindows.org/) or [wsl](https://learn.microsoft.com/en-us/windows/wsl/install) should be able to execute those scripts))

### How to use
The best source of documentation is the one provided by [Helix Editor](https://docs.helix-editor.com/). Also try Try typing :tutor + enter in normal mode for quick tutorial. There is no Kodiki-specific documentation as of yet, but it works just like Helix Editor with some tweaks and modifications, see below for the list of difference.

### Helix Editor highlights:
- Multiple selections
- [Tree-sitter](https://tree-sitter.github.io/tree-sitter/) integration
- [Language server](https://microsoft.github.io/language-server-protocol/) support
- Powerful code manipulation
- Built in Rust, for the terminal
- Modern builtin features
  
For more details please refer to [Helix Editor](https://helix-editor.com/)  

### In its current state Kodiki is built on:
- [Bevy Engine](https://github.com/bevyengine/bevy) as base platform for rendering, handling input, windows and everything platform dependent
- Modified [Helix Editor](https://github.com/helix-editor/helix) as a backend for text editor
- Simplified [WezTerm](https://github.com/wez/wezterm) as a backend for terminal emulation

Initial goal of the project was to explore the possibility of using [Bevy](https://bevyengine.org/) (a 3d game engine
in Rust with ECS) to write an IDE that will not only be functional but will also be pretty and capable of
things that are not very common or possible with classic frameworks that are used for making GUI applications.
(remark: 3d rendering is of course possible even without using Bevy or any 3d engine for that matter,
using Rust with Bevy is also very fun!)

There was no plan to use [Helix Editor](https://helix-editor.com/) in the beginning, initial intent was to borrow some code from it
for text processing but eventually Helix has proven to have a very mature, modular and lean codebase which allowed easy separation
of terminal-only code from other functions and using Bevy as a complete replacement for it.

In search for terminal emulation [WezTerm](https://wezfurlong.org/wezterm/index.html) was found. It has also undergone some changes
and simplifications for it to be embedded into Kodiki. Currently it is lacking a lot of its native functionality, but it is already
possible to use it for most cli applications, interact with shell in your OS, build your projects and use build logs to instantly jump to the file and line with error with ctrl+click
(for example cargo + mc + lazygit are working great already)

##### Kodiki has been used for its own development starting around June-July 2023

## Notable features
Even though the initial goal was to get a minimally functional application and then proceed with visuals I still managed to
squeeze in a few features that show off its platform capabilities just a tiny bit.

### Smooth camera movement
Had to be done. Current solution is not final, but already gives some satisfying smoothness.

Scrolling by pressing up/down arrow:  
[![scrolling with arrow keys](https://img.youtube.com/vi/IAY0eamPiY8/0.jpg)](https://www.youtube.com/watch?v=IAY0eamPiY8)

Scrolling with mouse wheel:  
[![scrolling with mouse wheel](https://img.youtube.com/vi/GYvCieO2Gu0/0.jpg)](https://www.youtube.com/watch?v=GYvCieO2Gu0)

### Minimap
Without it code navigation feels a little "blind" so it had to be implemented for MVP though in reality I found myself using it less frequently that I would expect.  
Featuring:  
Smooth camera movement while dragging viewport  
[![dragging minimap viewport](https://img.youtube.com/vi/xjnk5SFI-OQ/0.jpg)](https://www.youtube.com/watch?v=xjnk5SFI-OQ)

Smooth camera movement while clicking on random places in code  
[![clicking on minimap outside viewport](https://img.youtube.com/vi/IG9YUAhgGnk/0.jpg)](https://www.youtube.com/watch?v=IG9YUAhgGnk)

Preview of a code region that mouse cursor is hovering over  
[![minimap hovered region preview](https://img.youtube.com/vi/6YQ6o6K_lCM/0.jpg)](https://www.youtube.com/watch?v=6YQ6o6K_lCM)

Symbol bookmarks  
[![cached lsp symbols bookmarks on minimap](https://img.youtube.com/vi/-_Ddu9xR08s/0.jpg)](https://www.youtube.com/watch?v=-_Ddu9xR08s)

Diagnostics (errors, warnings, etc) and Git gutter (new lines - green, modified lines - orange)
(wip, you can make errors in your code and see the highlights in the meantime)

### Framerate management
To prevent your GPUs from meltdowns Kodiki stays in cool ~4fps mode when there is nothing going on.
If some action happens or animation needs to be played it spins up to 60 fps.
When camera is moving it switching to uncapped mode to avoid any jitter.
There is a debug tool for that currently in top left corner, if you click on it you'll see how it works.  
[![framerate manager](https://img.youtube.com/vi/medNhTYTBSc/0.jpg)](https://www.youtube.com/watch?v=medNhTYTBSc)

### Insert Mode overlay
In Vim and Helix I often find myself forgetting if I'm in insert mode or not. This overlay didn't solve the problem completely
but feels like a right first step.  
[![insert mode overlay](https://img.youtube.com/vi/BdIjtl_mRoU/0.jpg)](https://www.youtube.com/watch?v=BdIjtl_mRoU)

### Hovered Word highlight
Highlighting is a good way to show off some effects. Not yet spectacular, but bloom along with perspective distortion make things
pop out a little.  
[![hovered word highlight](https://img.youtube.com/vi/T8vC3zyQwQw/0.jpg)](https://www.youtube.com/watch?v=T8vC3zyQwQw)

### Go to file:line:column from terminal
The last feature in the MVP list I had initially. Later on I realized that diagnostics picker in Helix (space+d) is even a better way to
find out about errors in your code but this works regardless.  
[![go to file from terminal](https://img.youtube.com/vi/qK3DXPhVkZc/0.jpg)](https://www.youtube.com/watch?v=qK3DXPhVkZc)

### Themes
That's all goodies inherited from Helix Editor (WARNING! BLINKING LIGHTS ON VIDEO BELOW!)  
[![themes demo](https://img.youtube.com/vi/NkJ-eQZFLo4/0.jpg)](https://www.youtube.com/watch?v=NkJ-eQZFLo4)

### 3D Show-off
ctrl+home will bring you in free flight mode. Used for debugging but can also be used for cute selfies with your code  
[![3d show off](https://img.youtube.com/vi/siZVyxnc42E/0.jpg)](https://www.youtube.com/watch?v=siZVyxnc42E)

### Built-in Terminal
Building, building never changes  
[![building self in terminal](https://img.youtube.com/vi/cfv7_ew-ihU/0.jpg)](https://www.youtube.com/watch?v=cfv7_ew-ihU)

### What's next?

I'm taking a break from actively working on Kodiki, it's been more than a year(I started in August 2022, I think it was Bevy 0.8 back then),
I will work on something else for some time. But given that this is my primary IDE of choice now,
I doubt there will be no minor tweaks here and there.
After I'm back there is a lot of work to do and so many possibilities! Imagine having arsenal of all effects from video games in
you IDE! You can visualize everything, interact with it like in an fps, tps, rts! Record demos,
code in multiplayer, ditch files and folders and show everything as a graph! Visualize this graph and walk through it in first person!
Implement mods support and play Doom while you debug your app!
Meditate on your code while some shader draws you an infinite recursion of black holes eating each other in the background!

### Directions for near future development:

Finish missing terminal functionality (selection, copy/paste)
Polish of UI/UX with Helix
Workspaces (currently you can have only 1 folder open in Helix)
Multiple tabs in terminal
Experimenting with more effects and 3d

### Notable differences from Helix Editor
  
Added some functionality common for vs-like IDE-s:  
- Word selection by double click
- Selecting with shift + mouse click
- Enabled inlay hints by pressing ctrl+alt
- Added hints on mouse hover (initial implementation is already in Helix Editor, just needed small tweaking)
- Added minimap
- Added "current symbol under cursor" field in the status panel
- Added smart tab indent that matches expected indentation in current line

Other quality-of-life changes:  
- Added formatting to symbol picker
- Added insert mode overlay
- Added avoiding of showing the same completion if it was closed by user in the same cursor position
- Added initial focus on first completion item to apply it on first press of enter

### Hotkey changes

- Added selecting with shift+left/right/up/down/home/end in normal mode
- Added word deleting with control+del/backspace in normal mode
- Added control+/ for commenting
- Removed control+c for commenting
- Removed control+f/b for scrolling
- Added control+f for search
- Added control+c/v for copy/pasting in normal and insert modes
- Added control+tab for buffer picker. Might be not the best choice but works for a migrant from vs-like IDE-s
- Added alt+left/right for jumping forward/backward in jump list
- Added control+space for completion suggestions
- Added control+shift+space for signature help
- Added control+f for search
- Added enter for switching to insert mode

### Known Issues

#### Major
- There is no configuration apart from what comes along with Helix
- There is no selection in terminal
- There is no access to clipboard buffer in terminal
- Error messages can be too disruptive if there are more than 3 errors under cursor
- Error messages blend in with code too much visually
- Auto-complete can be too disruptive

#### Minor
- bevy_helix: Selecting multiple tabs highlights extra tab on the right
- bevy_helix: Highlights in rows with emojis have wrong offset because emojis are 2 chars long/wide
- bevy_helix: Completion takes to long to respond after lsp initialization is done
- bevy_helix: Zooming in hides soon to be invisible rows when they are still visible
- bevy_helix: Clicking in top/bottom part of viewport scrolls it(but disabling it can disable scrolling while selecting as in holding left mouse button and dragging it)
- bevy_helix: When syntax in file gets broken tabs will refuse to work
- bevy_helix: Cursor is missing in file/symbol picker
- bevy_helix: cfg code inactive diagnostics adds too much visual noise
- bevy_helix: Ctrl+hover on emojis will break them
- bevy_helix: Jumping between buffers will put viewport where cursor is, not where viewport was before the jump
- bevy_helix: Highlights don't work with inlay hints
- bevy_helix: On empty row pressing x selects one row below
- bevy_helix: ✘ symbol doesnt get rendered
- bevy_wezterm: Clicking outside terminal surface shouldnt register

There is an assorted list of tasks in TASKS.md that i might tackle eventually too

