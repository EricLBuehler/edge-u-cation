# edge-u-cation
Edge(u)cation: Cutting-edge multimodal LLMs on the edge with mistral.rs, using F8Q8

## F8Q8: 8-bit RTN-based blockwise nested quantization
F8E4M3 diagram:

<img src = https://github.com/user-attachments/assets/5de826f2-5c50-4a75-b9dd-f54b9e3e8d46 height = 75></img>

F8Q8 uses:
- Block size of 32
- 8-bit RTN quantization without any zero point/bias
- RTN scale $d$ is quantized to [F8E4M3](https://github.com/EricLBuehler/float8):

## Examples

Quadratic equation example:

<img src = https://github.com/user-attachments/assets/f41850c8-15f2-4997-a1ec-ea6694109022 height = 400></img>

Bridge design analysis example:

<img src = https://github.com/user-attachments/assets/67db5b2a-89a7-4898-952d-bfd85608bb88 height = 400></img>

