# LLMs on Edge: Novel Post-Training Quantization for Education Applications
Edge(u)cation: Cutting-edge multimodal LLMs on the edge with mistral.rs, using F8Q8.

Model used: https://huggingface.co/EricB/Phi-3.5-vision-instruct-UQFF.

**Abstract**:

Could a personalized, portable AI tutor transform education and improve outcomes for students in disadvantaged communities? Advancements in open-source large language models (LLMs), particularly multimodal models that can understand images and text, enable AI-driven personalized learning by giving students rapid and personalized feedback while addressing privacy concerns. However, running these models on consumer devices like a cell phone remains cost-prohibitive. I hypothesize that LLMs can be made more efficient for use on a phone through an improved algorithm that decomposes the parameters of a neural network into two parts, one of which has a certain range that we can exploit to reduce memory footprint. This would allow me to fit a powerful LLM onto a phone while retaining high accuracy. I found that my novel post-training quantization method reduces memory footprint for a cutting-edge 8 billion parameter model from 16 GB RAM to 8.16 GB RAM, a 49\% reduction in model size. Integrated into my custom inference engine written in Rust called mistral.rs, this approach powers Edge(u)cation, an AI tutor app I created for mobile devices. To validate its impact, I then deployed Edge(u)cation in several example settings including math and engineering education experiments through real-time, AI-driven feedback. In conclusion, this work demonstrates a scalable, cost-effective solution for personalized learning, fostering STEM engagement in under-resourced communities. I published all codes and models on GitHub in open source for anyone to access, use, and build on.

<img src = https://github.com/user-attachments/assets/4c575176-105c-47b2-a620-e8a008f7f135 height = 300></img>

Source: https://commons.wikimedia.org/wiki/File:Classroom_Picture_1.JPG

## F8Q8: 8-bit RTN-based blockwise nested quantization
F8E4M3 diagram:

<img src = https://github.com/user-attachments/assets/5de826f2-5c50-4a75-b9dd-f54b9e3e8d46 height = 75></img>

F8Q8:
- Uses a block size of 32
- Is a form of 8-bit RTN quantization without any zero point/bias
- Takes advantage of the observed range of the RTN scale $d$, to compress it into [F8E4M3](https://github.com/EricLBuehler/float8).

## Examples

Quadratic equation example:

<img src = https://github.com/user-attachments/assets/f41850c8-15f2-4997-a1ec-ea6694109022 height = 400></img>

Bridge design analysis example:

<img src = https://github.com/user-attachments/assets/67db5b2a-89a7-4898-952d-bfd85608bb88 height = 400></img>

