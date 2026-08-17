import 'package:flutter/material.dart';

import 'ui/studio_screen.dart';

void main() {
  runApp(const AutoDevStudioApp());
}

class AutoDevStudioApp extends StatelessWidget {
  const AutoDevStudioApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      debugShowCheckedModeBanner: false,
      title: 'AutoDev Studio',
      theme: ThemeData(useMaterial3: true),
      home: const StudioScreen(),
    );
  }
}
