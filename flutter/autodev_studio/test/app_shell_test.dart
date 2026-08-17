import 'package:autodev_studio/main.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  testWidgets('renders Studio title and disconnected state', (tester) async {
    await tester.pumpWidget(const AutoDevStudioApp());

    expect(find.text('AutoDev Studio'), findsOneWidget);
    expect(find.text('Disconnected'), findsOneWidget);
  });
}
