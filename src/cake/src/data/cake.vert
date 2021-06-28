#version 450

layout(location = 0) in vec2 Position;
layout(location = 1) in vec4 Color;

layout(location = 0) out vec4 fsin_Color;
layout(location = 1) out vec2 position;

void main()
{
    position = Position;
    gl_Position = vec4(Position * 2 - 1, 0, 1);
    fsin_Color = Color;
}