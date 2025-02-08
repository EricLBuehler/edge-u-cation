from huggingface_hub import HfApi

api = HfApi()
# Replace "username/model_id" with your model’s repo identifier
model_info = api.model_info("EricB/Phi-3.5-vision-instruct-UQFF", files_metadata=True)

# Each file is represented by a RepoFile object.
for repo_file in model_info.siblings:
    print(f"{repo_file.rfilename}: {repo_file.size} bytes")
