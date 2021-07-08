#version 450

struct KeyLocation {
    float left; 
    float right;
    int flags;
    int _;
};

layout(location = 1) in vec2 position;
layout(location = 2) in flat uint key;
layout(location = 3) in vec2 sides;
layout(location = 0) out vec4 fsout_Color;

layout (binding = 0) uniform UniformBuffer
{
    float width;
    float height;
    int start;
    int end;
    // int _keyCount;
};

// const int start = 0;
// const int end = 1505340;

const int keyCount = 128;

layout (binding = 1) readonly buffer BinaryTree
{
    ivec3 BinTree[];
};

// layout (binding = 2) readonly buffer Colors
// {
//     vec4 NoteColor[];
// };

// layout (binding = 3) readonly buffer Keys
// {
//     KeyLocation KeyLocations[];
// };

const float borderWidth = 0.0015;

ivec3 sampleAt(int pos) {
    return BinTree[pos];
}

ivec3 getNoteAt(uint key, int time) {
    int nextIndex = sampleAt(int(key)).x;

    int steps = 0;
    while(nextIndex > 0) {
        ivec3 node = sampleAt(nextIndex);
        if(time < node.x) nextIndex = node.y;
        else nextIndex = node.z;
        steps++;
    }

    ivec3 note = sampleAt(-nextIndex);

    return note;
}

bool midi_is_white(int p) {
  float k = fract(p * 5 / 12.0);
  return 0.1 < k && k < 0.55;
}

void main()
{
    int time = int(round(position.y * (end - start) + start));

    ivec3 note;

    note = getNoteAt(key, time);

    // fsout_Color = vec4(0, 0, 1, 1) / 10.0 * steps;

    if (note.z == -1) {
        discard;
    }

    // if (!midi_is_white(int(key))) {
    //     fsout_Color = vec4(1, 1, 1, 1);
    // } else {
    //     fsout_Color = vec4(0, 0, 0, 1);
    // }

    // return;

    float left = sides.x;
    float right = sides.y;

    int viewHeight = end - start;

    float distFromTop = float(note.y - time);
    float distFromBottom = float(time - note.x);

    float distFromLeft = float(position.x - left);
    float distFromRight = float(right - position.x);

    float vdist = min(distFromTop, distFromBottom) / viewHeight / width * height;
    float hdist = min(distFromLeft, distFromRight);

    float minDist = min(vdist, hdist);

    vec4 col = vec4((note.z & 0xFF) / 255.0, ((note.z >> 8) & 0xFF) / 255.0, ((note.z >> 16) & 0xFF) / 255.0, 1);

    if(minDist < borderWidth) {
        col.xyz *= 0.6;
    }
    fsout_Color = col;
}