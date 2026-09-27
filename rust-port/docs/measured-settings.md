# Measured settings: names, types, values, geometry

Everything here was read from the original's own files on this machine. Nothing
was modified. Where a value could not be read from the original it says so
instead of guessing.

## 1. The original's own `settings.json`

Read-only from the Proton prefix of appid 2420510:

```
/home/mila/.local/share/Steam/steamapps/compatdata/2420510/pfx/drive_c/users/steamuser/AppData/Local/HoloCure/settings.json
```

This is the authoritative source for field names and types. Types matter more
than values: six of the toggles are stored as **numbers**, not booleans.

### Numbers (`f64` in the port)

| Key | Value in the file |
|---|---|
| `musicVolume` | 0.4 |
| `soundVolume` | 0.6 |
| `attackAlpha` | 1.0 |
| `vibration` | 1.0 |
| `Resolution` | 1.0 |
| `hideFullHP` | 1.0 |
| `showSkillRadius` | 1.0 |
| `portDisplay` | 1.0 |
| `aboveHP` | 1.0 |
| `showHPVal` | 1.0 |
| `showHUDHP` | 1.0 |

`Resolution` is an index into the resolution list, not a pixel size.

### Booleans

`screenShake`, `lightFX`, `showStamps`, `hhMessages`, `fullscreen`,
`showDamageText`, `hiscoreName`, `hiscorenames`, `readyToStart`.

`hiscoreName` is a boolean despite its name. It is consent to publish this
user's name, and it is distinct from `hiscorenames`, which only controls whether
other players' names are displayed.

### Key lists

```json
"theButtons":         ["SPACE","SHIFT","A","D","W","S","CTRL"]
"controllerButtons":  ["gp_face1","gp_shoulderlb","gp_face2","gp_shoulderrb","gp_start","gp_select","gp_face3"]
```

Both carry **seven entries although the rooms only read indices 0..5**. The
seventh is written by the original and never read. The port keeps it.

`theButtons` stores key *names*; the live input manager holds virtual-key codes.
`controllerButtons` stores `gp_*` names; `controllerButtonList` holds codes:

| Code | Name | Code | Name |
|---|---|---|---|
| 32769 | `gp_face1` | 32775 | `gp_select` |
| 32770 | `gp_shoulderlb` | 32776 | `gp_padu` |
| 32771 | `gp_face2` | 32777 | `gp_padd` |
| 32772 | `gp_shoulderrb` | 32778 | `gp_padl` |
| 32773 | `gp_start` | 32779 | `gp_padr` |

32774 is `gp_face3` and is absent from `controllerButtonList`, which holds
`[32769, 32770, 32771, 32772, 32773, 32775, 32774, 32776, 32778, 32777]` - note
the non-sequential order.

### First-run defaults are NOT verified

The values above are what this machine's file happens to hold. The volumes
(0.4 / 0.6) may have been changed by the user. The port uses them as its
defaults and this note stays until a first-run capture proves otherwise.

## 2. `obj_Options` geometry, from its own `Draw` event

`container = [320, 48]` and the room runs at 640x360 while it is open.

| Element | Position |
|---|---|
| Panel sprite `hud_optionsmenu` | `container[0], container[1]` |
| Title `SETTINGS` | `container[0], container[1] + 10` (shadow at +13), centred |
| Row button `hud_OptionButton` | `container[0] + 12, container[1] + 43 + i*34` |
| Row icon | `container[0] - 98, container[1] + 56 + i*34` |
| Row label, left aligned | `container[0] - 66, container[1] + 51 + i*34` |
| Slider bar `hud_sliderBar` | `container[0] + 10, container[1] + 43 + i*34` |
| Slider knob `hud_slider` | `container[0] + 20 + value*70, container[1] + 56 + i*34` |
| Toggle `hud_toggleButton` | `container[0] + 81, container[1] + 56 + i*34` |
| Language value, centred | `container[0] + 52, container[1] + 51 + i*34`, arrows at `±38` |
| Resolution value, centred | `container[0] + 52, container[1] + 51 + i*34`, arrows at `±38` |
| `portDisplay` value, centred | `container[0] + 64, container[1] + 51 + i*34`, arrows at `-18` / `+38` |
| Scroll arrows `hud_scrollArrows` | `container[0] + 109, container[1] + 39` and `+ 269` |
| Scrollbar rectangle | `container[0] + 108 .. 110`, top `container[1] + 44 + scrollDist*range` |
| Scrollbar height | `1540 / maxOptions[page]`, step `220 / maxOptions[page]` |

`maxOptions = [7, 12]`, so the scrollbar step is 31.4 px on page 0 and 18.3 px
on page 1.

The knob position uses `value * 70` for the volumes and
`((attackAlpha - 0.3) / 0.7) * 70` for attack alpha, which is why the slider bar
is 70 px wide and why attack alpha's floor is 0.3.

Mouse hit test from `Draw` and `Mouse_53`: `MouseOverButton("long",
container[0] + 12, container[1] + 43 + i*34, screensize)` with `screensize = 1`
in `rm_Title` and `2` in `rm_PauseRoom`. `rm_Options` runs inside `rm_Title`.

### Keybinds submenu

Title `KEYBINDS` at the same place as `SETTINGS`. Icons `hud_keybindIcons` at
`container[0] - 98`. Labels left aligned at `container[0] - 66`. Key names right
aligned at `container[0] + 90, container[1] + 51 + i*34`. While a key is being
remapped the row is dimmed with a half-alpha black rectangle from
`container[0] + 50 .. 91`, `container[1] + 48 + i*34 .. 65 + i*34`.

### Controller submenu

Title `CONTROLLER`. `spr_brackets` at `228, 104`. Arrows at
`container[0] + 75 ± 18`, icon `hud_controllerButtonIcons` at
`container[0] + 75, container[1] + 56 + i*34`. `OK!` at
`container[0] + 80, container[1] + 254` once `controllerSet`.

## 3. `ValidKeysOnly` is empty

`script-ValidKeysOnly.gml` exists in the export but is **zero bytes**. The
decompiler recovered its call site in `Step_0` but not its body.

What is verified from the call site:

- the illegal keys are `[`, `]`, `/`, `\`, `;`, `WIN KEY`, `+`, `=`, `*`;
- a key already bound to one of the other five slots is rejected;
- an accepted key is written, `remapping` is cleared, `canControl = false` and
  `alarm[0] = 10`, then `SetKeyboardControls()` runs;
- `SaveSettings()` runs only once `setAll` reaches 0.

What is **not** verified: whatever else `ValidKeysOnly` rejects. The port
implements only the illegal list plus the duplicate check and marks the rest as
unverified rather than inventing a rule.

## 4. The `Resolution` quirk

`Create_0` reads the stored value into `selectedResolution`, calls
`GetCurrentController()`, and then ends with `global.Resolution = 1;`. The live
value is forced to 1280x720 while the pending one keeps whatever was stored, so
leaving the room without confirming leaves the stored value untouched.
