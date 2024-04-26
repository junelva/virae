### virae is a technical app engine.

```
This engine is not yet operational!
```

This is a personal game engine. The architecture is not designed for expansive, high-performance work. All project-specific content is expected to be initialized up-front. I can only work on this in my free time. The eventual goal is to make art. If you want to make use of it, feel free to reach out.

#### development log and planning.
```
[x] i am breaking and reshaping code.

what follows in this zone is random notetaking,
often out of order and simply used as scratch.

- [x] increase usage of `?` operator & expect
- [x] implement texture/sprite rendering
- [ ] blending modes for transparent textures
- [x] add texture to unit square? option?
- [ ] integrate an ecs - flax? survey.
- [x] api-ify the engine-y usage design.
- [ ] create example game using said api.

sunday's evening:
- [x] actually position rects.
- [x] split out loc/rot/scale.
- [x] eat chimken in 30 min.
- shader prep work?
  - [x] shade the ui rect?
  - [x] uv coords on vertices.
  - [x] pass screen res to fragment shader.
(... what am i even doing? keep forgetting...
  the rect locations need to be
  in pixel coordinates
  and i'm waffling working around
  that for two days now.
  maybe because i'm unsure of just
  how i want to work with
  these rects... absolute pixel size
  on screen? ideally?
  there's untread ground and
  unknown potential for issues.
... okay I did it. [x] )
- [x] debugging time: rect scale incorrect
- [ ] overlapping rect mask text.
  - [x] tie text to rect for testing.
  - [ ] depth buffer mask it.
- [ ] click'n drag rect.

saturday's docket:
- [x] trying to get ANY work done...
- [x] instancing needs to be implemented.
- [x] it may be fun to work on gui rendering.

friday's plan:
- coord system for placing gui elements
is screen pixels.
- [x] the render space unit square
for gui rects is (-0.5, -0.5)-(0.5, 0.5).
- [x] window pixel coords (0, 0) top left.
- [x] coords manipulated via the
rect's transform matrix.
- [x] therefore a function that translates a
       window pixel rect to a render space
       transformation matrix is required.
       [x] yay, math...

this may cause problems
later with high dpi screens.
i'll worry about that when
it becomes a problem.

such is life. [x]
got distracted, added shader hotloading.
meow. [x] mrr. [x]

thursday:
custom geometry was locked to view space.
> [x] update uniform on resize.
and convenience functions for manipulating
> [x] take a break.
4x4 matrices need to be developed in order
> [x] [x] [x] [x] eat a banana.
to control the... positioning of rectangles.
> [x] design coordinate system.

wednesday:
so far it uses glyphon for text. [x]
and wgpu for geometry rendering. [x]
in the interest of making a gui. [ ]
to composite 3D game art assets. [ ]
and sequence animations & audio. [ ]
```
