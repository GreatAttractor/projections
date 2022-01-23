//
// Map Projections
// Copyright (c) 2022 Filip Szczerek <ga.software@yahoo.com>
//
// This project is licensed under the terms of the MIT license
// (see the LICENSE file for details).
//

#version 330 core

in vec2 position;
out vec2 tex_coord;

void main()
{
    // apply texture coords (0,1)-(1,0) to unit quad (-1,-1)-(1,1)
    tex_coord.xy = position.xy / 2 + vec2(0.5, 0.5);

    gl_Position.x = position.x;
    gl_Position.y = -position.y;
    gl_Position.z = 0.0;
    gl_Position.w = 1.0;
}
