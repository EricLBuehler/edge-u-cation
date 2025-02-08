package com.example.mistralrs

import android.app.DownloadManager
import android.content.Context
import android.graphics.Bitmap
import android.graphics.ImageDecoder
import android.net.Uri
import android.os.Build
import android.os.Bundle
import android.os.Environment
import android.provider.MediaStore
import android.util.Log
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.activity.result.ActivityResultLauncher
import androidx.activity.result.PickVisualMediaRequest
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.Send
import androidx.compose.material.icons.filled.AddCircle
import androidx.compose.material.icons.filled.Send
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.layout.onSizeChanged
import androidx.compose.ui.platform.LocalConfiguration
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.navigation.NavController
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.rememberNavController
import com.example.mistralrs.ui.theme.MistralrsTheme
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.collect
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.launch
import java.io.File
import java.io.FileOutputStream
import java.io.InputStream
import java.net.HttpURLConnection
import java.net.URL
import kotlin.concurrent.thread
import kotlin.math.roundToInt

// The FFI class remains unchanged.
class FFI {
    companion object {
        init {
            System.loadLibrary("mistralrs_ffi")
        }

        external fun load_model(
            self: MainActivity,
            prompt: String,
            cfgPath: String,
            genCfgPath: String,
            uqffPath: String,
            preProcPath: String,
            procPath: String,
            resPath: String,
            tokPath: String,
            tokCfgPath: String
        ): String

        external fun run_vision(
            self: MainActivity,
            prompt: String,
            pixels: IntArray,
            width: Int,
            height: Int,
            cfgPath: String,
            genCfgPath: String,
            uqffPath: String,
            preProcPath: String,
            procPath: String,
            resPath: String,
            tokPath: String,
            tokCfgPath: String,
            cimatrixPath: String
        ): String

        external fun run_text(
            self: MainActivity,
            prompt: String,
            cfgPath: String,
            genCfgPath: String,
            uqffPath: String,
            preProcPath: String,
            procPath: String,
            resPath: String,
            tokPath: String,
            tokCfgPath: String,
            cimatrixPath: String
        ): String
    }
}

data class FileRef(val name: String, val size: Long)

enum class FileName {
    UQFF, RESIDUAL, CONFIG, PREPROCESSOR, PROCESSOR, GENERATION_CONFIG, TOKENIZER, TOKENIZER_CONFIG, CIMATRIX
}

val PHI3_5_UQFF_Q8_0: Map<FileName, FileRef> = mapOf(
    FileName.UQFF to FileRef("phi3.5-vision-instruct-q8_0.uqff", 3955041346L),
    FileName.RESIDUAL to FileRef("residual.safetensors", 848556296L),
    FileName.CONFIG to FileRef("config.json", 3778L),
    FileName.PREPROCESSOR to FileRef("preprocessor_config.json", 442L),
    FileName.PROCESSOR to FileRef("processor_config.json", 119L),
    FileName.GENERATION_CONFIG to FileRef("generation_config.json", 136L),
    FileName.TOKENIZER to FileRef("tokenizer.json", 1852867L),
    FileName.TOKENIZER_CONFIG to FileRef("tokenizer_config.json", 9519L),
    FileName.CIMATRIX to FileRef("collected-129.cimatrix", 2242713L)
)

val PHI3_5_UQFF_Q4K: Map<FileName, FileRef> = mapOf(
    FileName.UQFF to FileRef("phi3.5-vision-instruct-q4k.uqff", 2093851618L),
    FileName.RESIDUAL to FileRef("residual.safetensors", 848556296L),
    FileName.CONFIG to FileRef("config.json", 3778L),
    FileName.PREPROCESSOR to FileRef("preprocessor_config.json", 442L),
    FileName.PROCESSOR to FileRef("processor_config.json", 119L),
    FileName.GENERATION_CONFIG to FileRef("generation_config.json", 136L),
    FileName.TOKENIZER to FileRef("tokenizer.json", 1852867L),
    FileName.TOKENIZER_CONFIG to FileRef("tokenizer_config.json", 9519L),
    FileName.CIMATRIX to FileRef("collected-129.cimatrix", 2242713L)
)

class MainActivity : ComponentActivity() {
    private lateinit var pickMedia: ActivityResultLauncher<PickVisualMediaRequest>

    var outputTextState by mutableStateOf("")
    var isSubmitting by mutableStateOf(false)
    var bitmapState by mutableStateOf<Bitmap?>(null)
    var userInput by mutableStateOf("")

    var cfgPath by mutableStateOf("")
    var genCfgPath by mutableStateOf("")
    var uqffPath by mutableStateOf("")
    var preProcPath by mutableStateOf("")
    var procPath by mutableStateOf("")
    var resPath by mutableStateOf("")
    var tokPath by mutableStateOf("")
    var tokCfgPath by mutableStateOf("")
    var cimatrixPath by mutableStateOf("")

    // Download file function with progress as before.
    private fun downloadFileWithProgress(
        fileUrl: String, destinationFile: File, totalLength: Long
    ): Flow<Float> = flow {
        val url = URL(fileUrl)
        val connection: HttpURLConnection = url.openConnection() as HttpURLConnection
        connection.requestMethod = "GET"
        connection.connect()

        val inputStream: InputStream = connection.inputStream
        val outputStream = FileOutputStream(destinationFile)

        val data = ByteArray(4096)
        var count: Int
        var currentLength = 0L

        Log.d("Downloading", "Downloading from $fileUrl")
        while (inputStream.read(data).also { count = it } != -1) {
            currentLength += count
            outputStream.write(data, 0, count)
            emit(currentLength.toFloat() / totalLength.toFloat())
        }

        outputStream.close()
        inputStream.close()
    }.flowOn(Dispatchers.IO)

    @Composable
    fun DownloadProgressDialog(progress: Float, onCancel: () -> Unit) {
        AlertDialog(onDismissRequest = onCancel,
            title = { Text(text = "Downloading Model", fontWeight = FontWeight.Bold) },
            text = {
                Column(modifier = Modifier.fillMaxWidth()) {
                    LinearProgressIndicator(
                        progress = { progress },
                        modifier = Modifier.fillMaxWidth(),
                    )
                    Spacer(modifier = Modifier.height(8.dp))
                    Text(text = "${(progress * 100).toInt()}% downloaded")
                }
            },
            confirmButton = {
                TextButton(
                    onClick = onCancel, colors = ButtonDefaults.buttonColors(
                        containerColor = MaterialTheme.colorScheme.primary,
                        contentColor = Color.Black  // Force button text to be black
                    )
                ) { Text("Cancel") }
            })
    }

    private suspend fun downloadFiles(
        files: Map<FileName, FileRef>,
        repoUrl: String,
        cacheDir: File,
        onProgressUpdate: (Float) -> Unit,
        updatePaths: (FileName, String) -> Unit
    ) {
        for ((name, fileRef) in files) {
            val file = File(cacheDir, fileRef.name)
            if (file.length() != fileRef.size) {
                onProgressUpdate(0f)
                downloadFileWithProgress(
                    repoUrl + fileRef.name, file, fileRef.size
                ).collect { onProgressUpdate(it) }
            } else {
                onProgressUpdate(1f)
            }
            updatePaths(name, file.absolutePath)
        }
    }

    @OptIn(ExperimentalMaterial3Api::class)
    @Composable
    fun DownloadScreen(onDownloadComplete: () -> Unit) {
        // Using Scaffold with a TopAppBar gives a modern header
        Scaffold(topBar = {
            TopAppBar(
                title = { Text("Select Model", style = MaterialTheme.typography.titleLarge) },
                colors = TopAppBarDefaults.topAppBarColors(
                    containerColor = MaterialTheme.colorScheme.primaryContainer,
                    titleContentColor = MaterialTheme.colorScheme.onPrimaryContainer
                )
            )
        }) { innerPadding ->
            var showDialog by remember { mutableStateOf(false) }
            var progress by remember { mutableStateOf(0f) }
            val coroutineScope = rememberCoroutineScope()

            Column(
                modifier = Modifier
                    .padding(innerPadding)
                    .fillMaxSize()
                    .padding(16.dp),
                horizontalAlignment = Alignment.CenterHorizontally,
                verticalArrangement = Arrangement.Center
            ) {
                Button(
                    onClick = {
                        showDialog = true
                        val repoUrl =
                            "https://huggingface.co/EricB/Phi-3.5-vision-instruct-UQFF/resolve/main/"
                        val cacheDir: File = baseContext.cacheDir
                        coroutineScope.launch {
                            downloadFiles(PHI3_5_UQFF_Q8_0,
                                repoUrl,
                                cacheDir,
                                onProgressUpdate = { progress = it },
                                updatePaths = { name, path ->
                                    when (name) {
                                        FileName.CONFIG -> cfgPath = path
                                        FileName.UQFF -> uqffPath = path
                                        FileName.PROCESSOR -> procPath = path
                                        FileName.RESIDUAL -> resPath = path
                                        FileName.TOKENIZER -> tokPath = path
                                        FileName.PREPROCESSOR -> preProcPath = path
                                        FileName.TOKENIZER_CONFIG -> tokCfgPath = path
                                        FileName.GENERATION_CONFIG -> genCfgPath = path
                                        FileName.CIMATRIX -> cimatrixPath = path
                                    }
                                })
                            onDownloadComplete()
                        }
                    }, modifier = Modifier
                        .fillMaxWidth()
                        .padding(vertical = 8.dp)
                ) {
                    Text("Use large model (6GB)")
                }

                Button(
                    onClick = {
                        showDialog = true
                        val repoUrl =
                            "https://huggingface.co/EricB/Phi-3.5-vision-instruct-UQFF/resolve/main/"
                        val cacheDir: File = baseContext.cacheDir
                        coroutineScope.launch {
                            downloadFiles(PHI3_5_UQFF_Q4K,
                                repoUrl,
                                cacheDir,
                                onProgressUpdate = { progress = it },
                                updatePaths = { name, path ->
                                    when (name) {
                                        FileName.CONFIG -> cfgPath = path
                                        FileName.UQFF -> uqffPath = path
                                        FileName.PROCESSOR -> procPath = path
                                        FileName.RESIDUAL -> resPath = path
                                        FileName.TOKENIZER -> tokPath = path
                                        FileName.PREPROCESSOR -> preProcPath = path
                                        FileName.TOKENIZER_CONFIG -> tokCfgPath = path
                                        FileName.GENERATION_CONFIG -> genCfgPath = path
                                        FileName.CIMATRIX -> cimatrixPath = path
                                    }
                                })
                            onDownloadComplete()
                        }
                    }, modifier = Modifier
                        .fillMaxWidth()
                        .padding(vertical = 8.dp)
                ) {
                    Text("Use small model (3GB)")
                }
            }

            if (showDialog) {
                DownloadProgressDialog(progress = progress, onCancel = { showDialog = false })
            }
        }
    }

    @Composable
    fun DownloadOrDoNothing(navController: NavController) {
        var downloadCompleted by remember { mutableStateOf(false) }
        LaunchedEffect(downloadCompleted) {
            if (downloadCompleted) {
                navController.navigate("home") {
                    popUpTo("splash") { inclusive = true }
                }
            }
        }
        DownloadScreen(onDownloadComplete = { downloadCompleted = true })
    }

    @OptIn(ExperimentalMaterial3Api::class)
    @Composable
    fun UpdatingScreen() {
        // Using Scaffold with a TopAppBar on the home screen
        Scaffold(topBar = {
            TopAppBar(
                title = { Text("LLMs on Edge", style = MaterialTheme.typography.titleLarge) },
                colors = TopAppBarDefaults.topAppBarColors(
                    containerColor = MaterialTheme.colorScheme.primaryContainer,
                    titleContentColor = MaterialTheme.colorScheme.onPrimaryContainer
                )
            )
        }, bottomBar = {
            BottomAppBar(
                tonalElevation = 8.dp, modifier = Modifier.fillMaxWidth()
            ) {
                Row(
                    modifier = Modifier.padding(8.dp),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    // TextField occupies most of the width.
                    TextField(
                        value = userInput,
                        onValueChange = { newValue -> userInput = newValue },
                        modifier = Modifier.weight(0.7f),
                        placeholder = {
                            Text(
                                "Enter prompt here", fontSize = 14.sp,
                            )
                        },
                        enabled = !isSubmitting,
                        textStyle = TextStyle(
                            fontSize = 14.sp
                        ),
                    )
                    // Column for action buttons.
                    Column(
                        verticalArrangement = Arrangement.spacedBy(4.dp),
                        horizontalAlignment = Alignment.End,
                        modifier = Modifier
                            .weight(0.3f)
                            .padding(start = 8.dp)
                    ) {
                        OutlinedButton(
                            onClick = { pick_media() },
                            modifier = Modifier
                                .height(24.dp)
                                .align(Alignment.End),
                            contentPadding = PaddingValues(vertical = 0.dp, horizontal = 12.dp),
                            enabled = !isSubmitting,
                            colors = ButtonDefaults.buttonColors(
                                containerColor = MaterialTheme.colorScheme.primary,
                                contentColor = Color.Black  // Force button text to be black
                            )
                        ) {
                            Icon(
                                imageVector = Icons.Default.AddCircle,
                                contentDescription = "Add image"
                            )
                            Spacer(modifier = Modifier.width(4.dp))
                            Text("Image", fontSize = 12.sp)
                        }
                        OutlinedButton(
                            onClick = {
                                isSubmitting = true
                                start_inference()
                            },
                            modifier = Modifier
                                .height(24.dp)
                                .align(Alignment.End),
                            contentPadding = PaddingValues(vertical = 0.dp, horizontal = 12.dp),
                            enabled = !isSubmitting,
                            colors = ButtonDefaults.buttonColors(
                                containerColor = MaterialTheme.colorScheme.primary,
                                contentColor = Color.Black  // Force button text to be black
                            )
                        ) {
                            Icon(
                                imageVector = Icons.AutoMirrored.Filled.Send,
                                contentDescription = "Submit"
                            )
                            Spacer(modifier = Modifier.width(4.dp))
                            Text("Submit", fontSize = 12.sp)
                        }
                    }
                }
            }
        }) { innerPadding ->
            // Main content area
            val outputScrollState = rememberScrollState()
            LaunchedEffect(outputTextState) {
                outputScrollState.animateScrollTo(outputScrollState.maxValue)
            }
            val configuration = LocalConfiguration.current
            val screenHeight = configuration.screenHeightDp.dp
            val maxImageHeight = screenHeight * 0.3f

            Column(
                modifier = Modifier
                    .padding(innerPadding)
                    .fillMaxSize()
                    .verticalScroll(rememberScrollState())
                    .padding(16.dp),
                horizontalAlignment = Alignment.CenterHorizontally,
                verticalArrangement = Arrangement.spacedBy(16.dp)
            ) {
                bitmapState?.let { bmp ->
                    Image(
                        bitmap = bmp.asImageBitmap(),
                        contentDescription = "Updated image",
                        modifier = Modifier
                            .fillMaxWidth()
                            .heightIn(max = maxImageHeight)
                            .align(Alignment.Start)
                    )
                } ?: Text("No image selected", style = MaterialTheme.typography.bodyLarge)

                // Display the output text in a Surface for a card-like feel.
                Surface(
                    tonalElevation = 2.dp,
                    shape = MaterialTheme.shapes.medium,
                    modifier = Modifier.fillMaxWidth()
                ) {
                    Text(
                        text = outputTextState,
                        modifier = Modifier
                            .padding(12.dp)
                            .fillMaxHeight(0.7f)
                            .align(Alignment.Start),
                        style = MaterialTheme.typography.bodyMedium
                    )
                }
            }
        }
    }

    @Composable
    fun MyApp() {
        val navController = rememberNavController()
        NavHost(navController = navController, startDestination = "splash") {
            composable("splash") {
                DownloadOrDoNothing(navController = navController)
            }
            composable("home") {
                UpdatingScreen()
            }
        }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        setContent {
            MistralrsTheme {
                MyApp()
            }
        }

        pickMedia =
            registerForActivityResult(ActivityResultContracts.PickVisualMedia()) { uri: Uri? ->
                if (uri != null) {
                    Log.d("PhotoPicker", "Selected URI: $uri")
                    val selectedImage: Bitmap =
                        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.P) {
                            val source = ImageDecoder.createSource(contentResolver, uri)
                            ImageDecoder.decodeBitmap(source)
                        } else {
                            @Suppress("DEPRECATION") MediaStore.Images.Media.getBitmap(
                                contentResolver, uri
                            )
                        }
                    bitmapState = selectedImage
                } else {
                    Log.d("PhotoPicker", "No media selected")
                }
            }
    }

    fun pick_media() {
        pickMedia.launch(PickVisualMediaRequest(ActivityResultContracts.PickVisualMedia.ImageOnly))
    }

    fun start_inference() {
        if (bitmapState != null) {
            val pixelsArgb: Bitmap = bitmapState!!.copy(Bitmap.Config.ARGB_8888, false)
            val width = pixelsArgb.width
            val height = pixelsArgb.height
            val pixels = IntArray(width * height)
            pixelsArgb.getPixels(pixels, 0, width, 0, 0, width, height)

            thread {
                FFI.run_vision(
                    this,
                    userInput,
                    pixels,
                    width,
                    height,
                    cfgPath,
                    genCfgPath,
                    uqffPath,
                    preProcPath,
                    procPath,
                    resPath,
                    tokPath,
                    tokCfgPath,
                    cimatrixPath,
                )
                isSubmitting = false
            }
        } else {
            thread {
                FFI.run_text(
                    this,
                    userInput,
                    cfgPath,
                    genCfgPath,
                    uqffPath,
                    preProcPath,
                    procPath,
                    resPath,
                    tokPath,
                    tokCfgPath,
                    cimatrixPath,
                )
                isSubmitting = false
            }
        }
    }

    fun clear_output() {
        runOnUiThread { outputTextState = "" }
    }

    fun process_chunk(data: String) {
        if (data.contains("<|end|>").not()) {
            runOnUiThread { outputTextState += data }
        }
    }

    fun process_image_update(pixels: IntArray, width: Int, height: Int) {
        runOnUiThread {
            bitmapState = Bitmap.createBitmap(pixels, width, height, Bitmap.Config.ARGB_8888)
        }
    }
}
