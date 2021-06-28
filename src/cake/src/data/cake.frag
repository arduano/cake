#version 450

struct KeyLocation {
    float left; 
    float right;
    int flags;
    int _;
};

layout(location = 0) in vec4 fsin_Color;
layout(location = 1) in vec2 position;
layout(location = 0) out vec4 fsout_Color;

layout (binding = 0) uniform UniformBuffer
{
    int width;
    int height;
    int start;
    int end;
    int keyCount;
};

layout (binding = 1) readonly buffer BinaryTree
{
    ivec3 BinTree[];
};

layout (binding = 2) readonly buffer Colors
{
    vec4 NoteColor[];
};

layout (binding = 3) readonly buffer Keys
{
    KeyLocation KeyLocations[];
};

const float borderWidth = 0.0015;

ivec3 getNoteAt(int key, int time) {
    int nextIndex = BinTree[key].x;

    while(nextIndex > 0) {
        ivec3 node = BinTree[nextIndex];
        if(time < node.x) nextIndex = node.y;
        else nextIndex = node.z;
    }

    ivec3 note = BinTree[-nextIndex];

    return note;
}

void main()
{
    int testKey = int(floor(position.x * keyCount));

    int whiteKey = -1;
    int blackKey = -1;

    for (int i = 0; i < 9; i++) {
        int odd = i % 2;
        int o = (i - odd) / 2;
        if(odd == 1) o = -o;

        int k = testKey + o;
        if(k < 0 || k >= keyCount) continue;

        KeyLocation keyData = KeyLocations[k];
        if(keyData.left < position.x && keyData.right >= position.x) {
            if(keyData.flags == 1) blackKey = k;
            else whiteKey = k;
        }
    }

    int time = int(round(position.y * (end - start) + start));

    int key;
    ivec3 note;

    if(blackKey != -1) {
        note = getNoteAt(blackKey, time);
        key = blackKey; 
    }

    if(blackKey == -1 || note.z == -1) {
        note = getNoteAt(whiteKey, time);
        key = whiteKey; 
    }

    KeyLocation kdata = KeyLocations[key];

    float left = kdata.left;
    float right = kdata.right;

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

        if(minDist < borderWidth) {
            fsout_Color = NoteColor[note.z] * 0.6;
        }
        else {
            fsout_Color = NoteColor[note.z];
        }
    }
}