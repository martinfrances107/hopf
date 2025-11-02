# TODO

[] - Fix broken mesh ( bevy ) start and stop do not mesh.
     point2ply - shows the spacing is ONLY almost always even.

[] - smooth shading. Initial material issue

[] -Build a benchmark
    Newton Rahspson / Gradient descent is low.
    Could use binary search with brackets
    No path adjustment?


[] - One point on the on the bevy computed Mesh is very large
     dump the points and look for outlier.

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
