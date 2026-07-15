import pickletools

# Disassemble the file safely to see its raw blueprint instructions
with open("model.pkl", "rb") as f:
  pickletools.dis(f)
