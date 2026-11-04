import 'api_impl.dart' show ApiImpl;
import 'models/attachment.dart' show Attachment;

class AttachmentApi extends ApiImpl {
  const AttachmentApi({required super.client});

  Future<List<Attachment>> getAttachment() async {
    final response = await client.get('/attachments');
    final result = response.data as List<Map<String, dynamic>>;
    return result.map((data) => Attachment.fromJson(data)).toList();
  }

  Future<Attachment> createAttachment(
    String filename,
    String contentType,
  ) async {
    final response = await client.post(
      'attachments',
      data: {'filename': filename, 'contentType': contentType},
    );

    final result = response.data as Map<String, dynamic>;
    return Attachment.fromJson(result);
  }

  Future<Attachment> updateAttachment(String id, bool uploaded) async {
    final response = await client.post(
      '/attachments/$id',
      data: {'uploaded': uploaded},
    );

    final result = response.data as Map<String, dynamic>;
    return Attachment.fromJson(result);
  }

  Future<Attachment> deleteAttachment(String id, bool uploaded) async {
    final response = await client.delete(
      '/attachment/$id',
      data: {'uploaded': uploaded},
    );

    final result = response.data as Map<String, dynamic>;
    return Attachment.fromJson(result);
  }
}
