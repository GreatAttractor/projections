//
// Map Projections
// Copyright (c) 2022 Filip Szczerek <ga.software@yahoo.com>
//
// This project is licensed under the terms of the MIT license
// (see the LICENSE file for details).
//

//
// Discards triangles which are made too stretched or disjoint by the current projection.
//

#version 330 core

layout(triangles) in;
layout(triangle_strip, max_vertices = 3) out;

in VS_OUT
{
    vec2 tex_coord;
} gs_in[];

out GS_OUT
{
    vec2 tex_coord;
} gs_out;

// has to equal `DISCARD` in vertex shaders
const vec4 DISCARD = vec4(1.0e+9, 1.0e+9, 1.0e+9, 1.0e+9);

void main()
{
    vec4 v1 = gl_in[0].gl_Position;
    vec4 v2 = gl_in[1].gl_Position;
    vec4 v3 = gl_in[2].gl_Position;

    if (v1 == DISCARD || v2 == DISCARD || v3 == DISCARD)
    {
        return;
    }

    if (distance(v1, v2) > 1.0 || distance(v2, v3) > 1.0)
    {
        return;
    }

    gl_Position = v1;
    gs_out.tex_coord = gs_in[0].tex_coord;
    EmitVertex();
    gl_Position = v2;
    gs_out.tex_coord = gs_in[1].tex_coord;
    EmitVertex();
    gl_Position = v3;
    gs_out.tex_coord = gs_in[2].tex_coord;
    EmitVertex();
    EndPrimitive();
}
