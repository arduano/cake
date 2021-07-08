#version 450

layout(location = 0) in vec2 Position;
layout(location = 1) in vec2 Sides;
layout(location = 2) in uint Key;

layout(location = 1) out vec2 position;
layout(location = 2) out flat uint key;
layout(location = 3) out vec2 sides;

void main() {
  position = Position;
  sides = Sides;
  key = Key;
  gl_Position = vec4(Position * 2 - 1, 0, 1);
}