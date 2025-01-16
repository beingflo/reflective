export type Screen = 'app' | 'upload';

export type UpdateConfigRequest = {
  bucket: string;
  endpoint: string;
  region: string;
  access_key: string;
  secret_key: string;
};
