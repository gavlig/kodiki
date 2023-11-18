Autosave
Fullscreen F11
Continuous multiselect with pressed mouse button
Page up/down doesnt put cursor on top/bottom of file
Error report surface should have transparent backgound
Moving cursor after pasting puts it in the beginning of selection. Make it optional of find how to explicitly put it in the end
Insert closing pair of brackets or quotes only if there is empty space on the right
Entering insert mode within selection doesnt always have to put cursor in the beginning of that selection
Close both signature help and completion with esc
Copy/paste from OS (system clipboard commands are not bound to any key)
Copy/pasting experience is very frustrating
- Pasting yanked selection in the end of line makes a new line for some reason
- Paste where the helix cursor is when using middle mouse button
Search selected when pressing '/' (shift + * otherwise exists already)
Terminal: optionally scroll to end on output from pty
Bold text
Helix windows
Trailing spaces
Highlight token under text cursor
Smoother resizing
Mouse cursor should change when hovering over text
Clicking beyond line end should put cursor on the last symbol mainly to highlight opposite brackets
Show symbol bookmarks in editor surface too
Highlight hovered line in editor
- Mirror highlighted line in editor to minimap
Highlight hovered line in minimap
- Mirror highlighted line in minimap to editor
Emojis for minimap preview
Smooth scrolling in minimap preview
Highlight bookmarks/symbols in minimap preview
Highlight search results in minimap preview
Highlight diagnostics in minimap preview
Highlight selection in minimap preview
Reveal all symbols on minimap on demand
Show symbols when clicking on current symbol in top panel
Emulate inverted cell color on cursor like in Helix
When cursor is beyond visible part indicate it somehow (this one might not make too much sense)
Show in editor the target point of minimap travel too
Horizontal scrolling on touchpad
Page up/down via minimap or camera instead of Helix for consistency
Same as ctrl+click but on f12
f2 for rename refactor
Scroll with wheel in picker
Click with mouse in picker
Click with mouse in prompt
Show whitespaces when selected
Better auto indentation when writing a new {} scope
Clear selection on esc
Replace tabs with spaces and vice versa
Add borders to minimap
Smoother switching between buffers/documents
Calculate markdown doc size for autocomplete better to not occupy half of the screen
Add backlight to mouse cursor
Make highlighting selected word prettier(maybe animate coloring line as well?)
Sync cursos position with current line highlight
Fix stutter on minimap and panels when resizing
Unreal-like variables to debug different behavior without rebuilding (https://github.com/IyesGames/iyes_cli, https://github.com/RichoDemus/bevy-console, https://docs.rs/clap/latest/clap/)
Utilize "vector shapes" for some pretty graphics (https://github.com/james-j-obrien/bevy_vector_shapes)
Too many errors can cover all editor rendering it unusable (try using async on system with app for example)
Utilize Helix's should_render to avoid extra cpu usage and unnecessary updates
Show insert mode near cursor too
Quick way of closing buffer when browsing buffer list
":" menu puts last command when opened but it is indistinguishable for normally typed command. Better formalize what can and cant be done with it
When lsp is taking too long there has to be some indication.
When lsp is taking too long the other menues that were open should get closed maybe when results finally come in
Multiple terminals
Quick window/view to the top of file to add another "use"
Mark clicked paths in terminal with a different color
Support multiline paths in terminal
Indicate that terminal has scrolling/scrolled
Dont send pageup + pagedown to pty?
Apply colors for symbols in symbol picker
Highlight "owner" symbol for the file in symbol picker (like text_surface.rs -> TextSurface)
Helix: When deleting \n don't append all tabs from the next line and instead append starting with text on the line
Search in terminal
Make tonemapping optional
Support themes in wezterm
Apply same delays for keyboard input as in bevy_helix for bevy_wezterm
Helix: Use different color for error message surface background color
Terminal: handle ctrl+d
Terminal: Minimap!
Terminal: Scrolling as in bevy_helix
Helix: Delete all selected with del in insert mode too
Helix: Smooth color transition for pleasant theme switching
Helix: Remove extra cursors by another click on them
Tests

Refactor:

Use kodiki_ui for bevy_helix
Parallelize update_rows in SurfaceBevy (or maybe dont since it's likely that we will directly use text without surfaces later on)
is_highlighted in WordDescription should be replaced by a Component like with hovered words highlighting
Use the same scrolling mechanism as in minimap for goto_definition in the same file
Bake tree sitter node id into every spawned word to avoid looking it up every time we need it
Use types instead of long lists of arguments in systems that spawn stuff in surface and minimap (see example of such types in bevy_rapier system writeback etc)
Split minimap::update_transform into smaller systems
Word spawning should update only when either document version changes or viewport
Last helix frame clearing might not be necessary or at least not that way
Research async for input and tokio events
Implement Selection Search Highlight and Search Highlight on Helix side to avoid having extra logic in Kodiki
Remove row_index_wcache and other cache related indices from SurfaceCoords and WordDescription
When scrolling, in camera target row stays for 2-3 frames when holding down arrow making dy fluctuate between 0.035 and 0.047. Maybe we should aim for more stable dy
Instead of checking for app.should_close in every system do this once and make sure other systems wont suffer by strict scheduling
Split minimap::systems::input_mouse
Remove bevy_helix from context_switcher systems
Untie TextSurface from BevyReaderCamera
Rewrite SurfaceBevy with TextSurface
Remove animations_cleanup_components and do a cleaner solution instead
Reimplement scroll in wezterm_portable using Screen::stable_row_index_offset
Refactor field visibilities in bevy_ab_glyph structs
Make a dedicated system for text spawning because it's painful to drag all caches and assets into every system
Rename text_surface::word to cluster or chunk or string mesh because word is misleading
Make highlighting like buttons in context switcher a generic system like with hints

