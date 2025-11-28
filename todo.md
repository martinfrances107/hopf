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

[] - Fix smooth shading.
     when I go smooth why does the surface appear blemish
     is it becasue the points indexing is bad?

[] - Blender renders need to be redone, now that the fibre projection has been corrected.
