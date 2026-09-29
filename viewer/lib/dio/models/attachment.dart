class Attachment {
  final String id;
  final String key;
  final bool uploaded;
  final String contentType;
  final String filename;
  final String publicUrl;
  final String organization;
  final String createdAt;
  final Map<String, dynamic>? metadata;

  const Attachment({
    required this.id,
    required this.key,
    required this.uploaded,
    required this.filename,
    required this.publicUrl,
    required this.metadata,
    required this.createdAt,
    required this.contentType,
    required this.organization,
  });

  factory Attachment.fromJson(Map<String, dynamic> json) {
    return switch (json) {
      {
        'id': String id,
        'key': String key,
        'uploaded': bool uploaded,
        'filename': String filename,
        'publicUrl': String publicUrl,
        'created_at': String createdAt,
        'content_type': String contentType,
        'organization': String organization,
        'metadata': Map<String, dynamic>? metadata,
      } =>
        Attachment(
          id: id,
          key: key,
          metadata: metadata,
          uploaded: uploaded,
          filename: filename,
          publicUrl: publicUrl,
          createdAt: createdAt,
          contentType: contentType,
          organization: organization,
        ),
      _ => throw const FormatException('Failed to load attachment.'),
    };
  }
}
