ALTER TABLE tag 
  ADD CONSTRAINT tag_description_lowercase_ck
  CHECK (description = lower(description));