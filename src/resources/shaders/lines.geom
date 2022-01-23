//
// Map Projections
// Copyright (c) 2022 Filip Szczerek <ga.software@yahoo.com>
//
// This project is licensed under the terms of the MIT license
// (see the LICENSE file for details).
//

//
// Discards lines which are made too stretched or disjoint by the current projection.
//

#version 330 core

layout(lines) in;
layout(line_strip, max_vertices = 2) out;

// has to equal `DISCARD` in vertex shaders
const vec4 DISCARD = vec4(1.0e+9, 1.0e+9, 1.0e+9, 1.0e+9);

void main()
{
    vec4 v1 = gl_in[0].gl_Position;
    vec4 v2 = gl_in[1].gl_Position;

    if (v1 == DISCARD || v2 == DISCARD)
    {
        return;
    }

    bool line_too_stretched_or_disjoint = distance(v1, v2) > 0.2;
    if (line_too_stretched_or_disjoint)
    {
        return;
    }

    gl_Position = v1;
    EmitVertex();
    gl_Position = v2;
    EmitVertex();
    EndPrimitive();
}
