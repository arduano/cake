#version 450

struct KeyLocation {
    float left; 
    float right;
    int flags;
    int _;
};

layout(location = 1) in vec2 position;
layout(location = 0) out vec4 fsout_Color;

layout (binding = 0) uniform UniformBuffer
{
    int width;
    int height;
    int _start;
    int _end;
    int _keyCount;
};

const int start = 0;
const int end = 1505340;

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

int getNoteAt(int key, int time) {
    int nextIndex = BinTree[key].x;

    int steps = 0;
    while(nextIndex > 0) {
        ivec3 node = BinTree[nextIndex];
        if(time < node.x) nextIndex = node.y;
        else nextIndex = node.z;
        steps++;
    }

    ivec3 note = BinTree[-nextIndex];

    return steps;
}

bool midi_is_white(int p) {
  float k = fract(p * 5 / 12.0);
  return 0.1 < k && k < 0.55;
}

void main()
{
    // int testKey = int(floor(position.x * keyCount));

    // int whiteKey = -1;
    // int blackKey = -1;

    // for (int i = 0; i < 9; i++) {
    //     int odd = i % 2;
    //     int o = (i - odd) / 2;
    //     if(odd == 1) o = -o;

    //     int k = testKey + o;
    //     if(k < 0 || k >= keyCount) continue;

    //     KeyLocation keyData = KeyLocations[k];
    //     if(keyData.left < position.x && keyData.right >= position.x) {
    //         if(keyData.flags == 1) blackKey = k;
    //         else whiteKey = k;
    //     }
    // }

    int time = int(round(position.y * (end - start) + start));

    // int key;
    int key = int(floor(position.x * keyCount));
    ivec3 note;

    int steps = getNoteAt(key, time);
    fsout_Color = vec4(0, 0, 1, 1) / 10.0 * steps;

    // if (!midi_is_white(key)) {
    //     fsout_Color = vec4(1, 1, 1, 1);
    // } else {
    //     fsout_Color = vec4(0, 0, 0, 1);
    // }

    return;

    // if(blackKey == -1 || note.z == -1) {
    //     note = getNoteAt(whiteKey, time);
    //     key = whiteKey; 
    // }

    // KeyLocation kdata = KeyLocations[key];

    float left = 1.0 / keyCount * key;
    float right = 1.0 / keyCount * (key + 1);

    if(note.z == -1) {
        fsout_Color = vec4(0, 0, 0, 1);
    }
    else {
        int viewHeight = end - start;

        float distFromTop = float(note.y - time);
        float distFromBottom = float(time - note.x);

        float distFromLeft = float(position.x - left);
        float distFromRight = float(right - position.x);

        float vdist = min(distFromTop, distFromBottom) / viewHeight / width * height;
        float hdist = min(distFromLeft, distFromRight);

        float minDist = min(vdist, hdist);

        vec4 col = vec4(0, 0, 1, 1);

        if(minDist < borderWidth) {
            fsout_Color = col * 0.6;
        }
        else {
            fsout_Color = col;
        }
    }
}