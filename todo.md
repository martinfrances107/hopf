# TODO


Fix issue with dial -- 
  color with alpha should be used for decal 
  -- red channel used for height input into displacement node 
  -- Set aspect ratio to be 10 viewBox w=360, h=36 output 1200x120
  BUGS
    Decal is not working 
    Black text is not insert
    Why is color black ... not orange as set by the decals color mixer.

[] - Indicator sphere in blender
   -- learn gltf loading
   Want Red for norhern hemisphere
   Blue for southen Hemispahere

   -- apply clear coat material
   -- define materials in gltf along with (unused in blender brighter versiosn)

[] - select handle based on hovering proximity.

[] - indicator color --- anistropic brush steel

[] - when hovering over indicator ball -- transition to brighter colors.


[] - Bevy App user inputs 2 points to form lines

    - handles are defined as upside down triangle.

    - on DragStart take the translation of the selected handle.
      keep  lat, lon in a state variable and in dag update the rotation to point onto the base sphere. ]

    -- get a mesh from a line segment on the sphere to render
    -- mesh is rendered as a object next to the gismo.
    -- next  1 point in fixed, as the control point moved
       then the mesh is updated.
    --


[] - Fix broken mesh ( bevy ) start and end do not mesh.
     point2ply - shows the spacing is ONLY almost always even.

    HAVE STOP GAP FIX:

   The temp fix is to change the number of points per loop
    from 80 to 40. This hides the issue .. I need a general solution
    ( with tests )

[] - Fix smooth shading.
     when I go smooth why does the surface appear blemish
     is it becasue the points indexing is bad?

[] -Build a benchmark
    Newton Rahspson / Gradient descent is low.
    Could use binary search with brackets
    No path adjustment?

[] - Bevy app what is need to make a bevy plugin
    plugin implements HopfMeshbuilder ( copied from SphereMeshBuilder)
    .compute_mesh(u, v ) -> Result<Mesh,_> -- yields a Mesh object

[] - Blender renders need to be redone, now that the fibre projection has been corrected.
