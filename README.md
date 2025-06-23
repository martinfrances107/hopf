# Hopf fibration library

Contains utilities for visualizing the hopf fibration.

It is toolset used for generating point cloud (.ply ) and OBJ fils (*.obj )

This is currently a work in progress.

## The fibre

For a point ($\theta$, $\phi$) on the 2-sphere

The circle of the fibre lies on the 3-sphere

The fibre, F($\alpha$) = ($X_0$, $X_1$, $X_4$, $X_4$) ∈ $R^4$

where $\alpha$ ∈ [0,4π]

Here is the mapping.

$X_0$ = cos($\frac{\alpha + \phi}{2}$) sin($\frac{\theta}{2}$)

$X_1$ = sin($\frac{\alpha + \phi}{2}$) sin($\frac{\theta}{2}$)

$X_2$ = cos($\frac{\alpha - \phi}{2}$) cos($\frac{\theta}{2}$)

$X_3$ = sin($\frac{\alpha - \phi}{2}$) cos($\frac{\theta}{2}$)

## Stereographic Projection

The points on the hyper-sphere are commonly projected using stereographic projection

($X_0$, $X_1$, $X_4$, $X_4$) -> (x,y,z)

where

$x$ = $\frac{X_0}{1-X_3}$

$y$ = $\frac{X_1}{1-X_3}$

$z$ = $\frac{X_2}{1-X_3}$
