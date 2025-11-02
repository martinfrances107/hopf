# TODO

[] - Fix broken mesh ( bevy ) start and end do not mesh.
     point2ply - shows the spacing is ONLY almost always even.

    HAVE STOP GAP FIX:

   The temp fix is to change the number of points per loop
    from 80 to 40. This hides the issue .. I need a general solution
    ( with tests )


[] - Fix smooth shading.

[] -Build a benchmark
    Newton Rahspson / Gradient descent is low.
    Could use binary search with brackets
    No path adjustment?

[] - Bevy App user inputs 2 points to form lines
      each line produces a mesh.
    -- get a mesh from a line segment on the sphere to render
    -- mesh is rendered as a object next to the gismo.
    -- next  1 point in fixed, as the control point moved
       then the mesh is updated.
    --

[] - Bevy app what is need to make a bevy plugin
    plugin implements HopfMeshbuilder ( copied from SphereMeshBuilder)
    .compute_mesh(u, v ) -> Result<Mesh,_> -- yields a Mesh object

[] - Blender renders need to be redone, now that the fibre projection has been corrected.
