### virae is a technical app engine.
`This engine is not yet operational!`
I can only work on this in my free time.
The eventual goal is to make art.

#### development log and planning.
```
i am breaking and reshaping code. [x]

what follows in this zone is random notetaking,
often out of order and simply used as scratch.

saturday's docket:
  - [x] trying to get ANY work done...
  - [x] instancing needs to be implemented.
  - [x] it may be fun to work on gui rendering.

friday's plan:
  - the coordinate system for placing gui elements is screen pixels.
  - [x] the render space unit square for gui rects is (-0.5, -0.5)-(0.5, 0.5).
  - [x] window pixel coordinates use (0, 0) top left.
  - [x] these coordinates are manipulated via the rect's transform matrix.
  - [ ] therefore a function that translates a window pixel rect to a
         render space transformation matrix is required. [ ] yay, math...

this may cause problems down the line with high dpi screens.
but i'll worry about that problem when it becomes a problem.

such is life. [x] i got distracted and added shader hotloading.
meow. [ ] mrr. [x]

thursday:
custom geometry was locked to view space.    > [x] update uniform on resize.
and convenience functions for manipulating   > [x] take a break.
4x4 matrices need to be developed in order   > [x] [x] eat a banana.
to control the... positioning of rectangles. > [x] design coordinate system.

wednesday:
so far it uses glyphon for text. [x]
and wgpu for geometry rendering. [x]
in the interest of making a gui. [ ]
to composite 3D game art assets. [ ]
and sequence animations & audio. [ ]
```
