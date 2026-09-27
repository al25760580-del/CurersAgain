# Measured native UI

All coordinates are logical (640x360). Captured from the Steam build with a
passive inspector; only the cursor was moved, no destructive action confirmed.

## Hitboxes

`MouseOverButton` forces its scale argument to 1:

| Type | Region |
|---|---|
| `short` | `[x-45,x+45) x [y-15,y+15)` (90x30) |
| `long` | `[x-90,x+90) x [y,y+29)` (180x29) |
| `arrow` | `[x-10,x+10) x [y-10,y+10)` (20x20) |

Verified on the left version arrow: 230 and 249 are inside, 229 and 250 outside;
94 and 113 inside, 93 and 114 outside.

## Scores menu

| Element | Measurement |
|---|---|
| Right menu buttons | `menu_hiscoreButtons` at `(560, 65+34i)`, frame 0 idle / 1 selected |
| Settings button | `menu_hiscoreSettingsButton` at `(560,277)`, frame 0 |
| Right menu labels | `(560, 60+34i)`, white; black when the row is selected |
| `< >` arrows | `hud_scrollArrows2` frames 0/1 at `(500,y)` and `(620,y)`, **rows 1..3 only** |
| Top-3 containers | `menu_first` frames 0,1,2 at `(105, 50+58i)` |
| Other containers | `menu_scorecontainer` at `(285, 50+58i)` and `(105/285, 224+58i)` |
| Portrait | `spr_Title_Ame` at `(75,350)` |
| Name | `(117, y+5)` |
| Score | `(117, y+20)` |
| Time and level | `(117, y+36)` |
| `Rank` label | `(272, y+4)` left column, `(452, y+4)` right column |
| Rank number | `(272, y+17)` / `(452, y+17)` |
| Page numbers | `1` at `(193,334)`, the rest at `(193+20i,341)` |
| Footer | `CONFIRM: SPACE/ENTER | CANCEL: SHIFT/ESC` at `(630,345)` |
| Title | `- ALL TIME -` at `(285,15)` |

Row texts use `draw_text_scribble` with colour markup, e.g.
`[c_yellow]Fukurowl[/color]`.

## Settings

| Element | Measurement |
|---|---|
| Panel | `hud_optionsmenu` at `(320,48)` |
| Heading | black outline at `(320,61)`, white at `(320,58)` |
| Rows | `hud_OptionButton` frame 0/1 at `(332, 91+34i)`; `Quit` at `y=295` |
| Labels | `(254, 99+34i)`; white, black when selected, **`DELETE SCORES` red (255)** |
| Icons | `hud_hiscoreOptionIcons` frames `0,2,4,6,8` idle and `+1` selected, at `(222, 104+34i)` |
| Toggles | `hud_toggleButton` frame 1 at `(401,172)` and `(401,206)` |
| Version arrows idle | `hud_scrollArrows2` frames 0/1 at `(240,104)` and `(424,104)` |
| Version arrows selected | `hud_scrollArrows3` frames 0/1 at `(236,104)` and `(428,104)` |
| Arrow hitbox | 20x20 anchored at the **unselected** position, even when the sprite moves |

First row text: `LEADERBOARD: 0.7` at `(254,99)` with font asset 18.

**Hover selects the row**: moving the cursor over a row sets
`currentSettingsOption` without a click.

## Rename modal

| Element | Measurement |
|---|---|
| Panel | `spr_option_widebox` at `(320,130)` |
| Prompt | `Change Your Username:` outlined at `(320,145)` |
| Field | username at `(239,180)` |
| Buttons | `(270,215)` and `(370,215)` |
| Idle sprite | `hud_unselectButton` (69x21, origin 34,10) |
| Hovered or selected sprite | `hud_confirmButton` (89x29, origin 44,14) |
| Texts | `Confirm` at `(270,210)`, `Cancel` at `(370,210)` |
| Cancel | `Shift+Esc`; `Esc` alone does not close the modal |

Hover experiment:

| Step | Cursor | `renameOption` | Confirm sprite | Cancel sprite |
|---|---|---|---|---|
| Neutral | `(100,50)` | -1 | `hud_unselectButton` | `hud_unselectButton` |
| Over Confirm | `(270,215)` | 0 | `hud_confirmButton` | `hud_unselectButton` |
| Over Cancel | `(370,215)` | 1 | `hud_unselectButton` | `hud_confirmButton` |
| Back to neutral | `(100,50)` | 1 | `hud_unselectButton` | `hud_confirmButton` |

The large sprite follows the cursor and the selection persists after leaving.
While the field accepts text (`canType`) the game parks the system cursor, so
hover cannot be driven with the physical mouse in that state.

## Delete modal

| Element | Measurement |
|---|---|
| Panel | `hud_quitConfirm` at `(320,120)` |
| Title | `DELETE SCORES` at `(320,133)` |
| Buttons | `(320,177)` and `(320,207)` |
| Texts | `Yes` at `(320,171)`, `No` at `(320,201)` |
| Initial selection | `deleteSelect = 1` (No) |

Hover changes the drawn sprite but `deleteSelect` does not follow the cursor.
`Up`, `Down`, `Left` and `Right` produced no observed change of `deleteSelect`.
