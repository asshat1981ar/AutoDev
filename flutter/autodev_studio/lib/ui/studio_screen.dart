import 'package:flutter/material.dart';

class StudioScreen extends StatelessWidget {
  const StudioScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('AutoDev Studio')),
      body: Center(
        child: Semantics(
          label: 'AutoDev connection status: disconnected',
          child: const Text('Disconnected'),
        ),
      ),
    );
  }
}
