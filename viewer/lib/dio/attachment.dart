import 'apl_impl.dart';
import 'models/attachment.dart';

class AttachmentApi extends ApiImpl {
  const AttachmentApi({required super.client});

  Future<List<Attachment>> getAttachment() async {
    final response = await client.get('/attachments');
    if (response.statusCode == 200) {
      final result = response.data as List<Map<String, dynamic>>;
      return result.map((data) => Attachment.fromJson(data)).toList();
    } else {
      throw Exception(response.data);
    }
  }

  Future<Attachment> createAttachment(
    String filename,
    String contentType,
  ) async {
    final response = await client.post(
      'attachments',
      data: {'filename': filename, 'contentType': contentType},
    );

    if (response.statusCode == 200) {
      final result = response.data as Map<String, dynamic>;
      return Attachment.fromJson(result);
    } else {
      throw Exception(response.data);
    }
  }

  Future<Attachment> updateAttachment(String id, bool uploaded) async {
    final response = await client.post(
      '/attachments/$id',
      data: {'uploaded': uploaded},
    );

    if (response.statusCode == 200) {
      final result = response.data as Map<String, dynamic>;
      return Attachment.fromJson(result);
    } else {
      throw Exception(response.data);
    }
  }

  Future<Attachment> deleteAttachment(String id, bool uploaded) async {
    final response = await client.delete(
      '/attachment/$id',
      data: {'uploaded': uploaded},
    );

    if (response.statusCode == 200) {
      final result = response.data as Map<String, dynamic>;
      return Attachment.fromJson(result);
    } else {
      throw Exception(response.data);
    }
  }
}
