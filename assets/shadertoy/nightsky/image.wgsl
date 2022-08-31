// Something like the Night Sky by Zi7ar21 --- February 1st, 2020
// Updated February 1st, 17:15 Mountain Time

// If you didn't find this on Shadertoy, the original can be found at:
// https://www.shadertoy.com/view/ttcfRH

//
// This is a somewhat Night-Sky like scene. It is rendered in 2D, and is NOT physically accurate.
// I didn't take into account proper gas colors, gas distribution, etc.
// Also the stars have a Blackbody pallete but the probability of a random sampled star's
// color and luminance is not based on something accurate like Hertzsprung-Russell.
//

// Hashes https://www.shadertoy.com/view/4djSRW

fn hash11(p_in : f32) -> f32 {
    var p : f32 = fract(p_in * 0.1031);
    p *= p + 33.33;
    p *= p + p;
    return fract(p);
}

fn hash12(pos : vec2<f32>) -> f32 {
	var p3 : vec3<f32> = fract(vec3<f32>(pos.xyx) * 0.1031);
    var p33 : vec3<f32> = p3 + vec3<f32>(dot(p3, p3.yzx + 33.33));
    return fract((p33.x + p33.y) * p33.z);
}

fn hash13(p3_in : vec3<f32>) -> f32 {
	var p3 : vec3<f32> = fract(p3_in * 0.1031);
    p3 += vec3<f32>(dot(p3, p3.zyx + 31.32));
    return fract((p3.x + p3.y) * p3.z);
}

fn hash33(p3_in : vec3<f32>) -> vec3<f32> {
	var p3 : vec3<f32> = fract(p3_in * vec3<f32>(0.1031, 0.1030, 0.0973));
    p3 += vec3<f32>(dot(p3, p3.yxz + 33.33));
    return fract((p3.xxy + p3.yxx) * p3.zyx);
}

fn noise1(n : vec2<f32>) -> f32 {
    var b : vec4<f32> = vec4<f32>(floor(n), ceil(n)); 
    var f : vec2<f32> = smoothstep(vec2<f32>(0.0), vec2<f32>(1.0), fract(n));
	return mix(mix(hash12(b.xy), hash12(b.zy), f.x), mix(hash12(b.xw), hash12(b.zw), f.x), f.y);
}

fn modulo(x : f32, y : f32) -> f32 {
    return x - y * floor(x/y);
}

fn octaves() -> i32 {
    return 8;
}

// Atmospheric Distortion "Twinkling"
fn noise2(coord : vec2<f32>, n : f32) -> f32 {
    var componenta : f32 = hash13(vec3(coord, round(n - 0.5)));
    var componentb : f32 = hash13(vec3(coord, round(n + 0.5)));
    var componentc : f32 = mix(componenta, componentb, modulo(n, 1.0));
    return componentc;
}

// FBM Terrain Line
fn noise3(coord : f32) -> f32 {
    var componenta : f32 = hash11(round(coord - 0.5));
    var componentb : f32 = hash11(round(coord + 0.5));
    return mix(componenta, componentb, modulo(coord, 1.0));
}

// Color Offset, as Reccomended by user "elenzil"
// https://www.shadertoy.com/user/elenzil
fn colorednoise(coord : vec2<f32>, n : f32) -> vec3<f32> {
    var componenta : vec3<f32> = hash33(vec3<f32>(coord, round(n - 0.5)));
    var componentb : vec3<f32> = hash33(vec3<f32>(coord, round(n + 0.5)));
    var componentc : vec3<f32> = mix(componenta, componentb, modulo(n, 1.0));
    return componentc;
}

// FBM https://www.shadertoy.com/view/3dSBRh
// #define octaves 8
fn fbm2(x_in : vec2<f32>) -> f32 {
	var v : f32 = 0.0;
	var a : f32 = 0.4;
    var x : vec2<f32> = x_in;
	for(var i : i32 = 0; i < octaves(); i++) {
		v += a * noise1(x);
		x  = x * 2.0;
		a *= 0.6;
	}

	return v;
}

fn fbm1(x_in : f32) -> f32 {
	var v : f32 = 0.0;
	var a : f32 = 0.5;
    var x : f32 = x_in;
	for(var i : i32 = 0; i < octaves(); i++) {
		v += a * noise3(x);
		x  = x * 2.0;
		a *= 0.5;
	}

	return v;
}

// Blackbody Coloration (Made into a Function by LoicVDB)
// https://www.shadertoy.com/view/4tdGWM
fn blackbody(temperature : f32) -> vec3<f32> {
    var O : vec3<f32> = vec3<f32>(0.0);

    for(var i : f32 = 0.0; i < 3.0; i += 0.1) {
        var f : f32 = 1.0 + 0.5 * i;

        O[i32(i)] += 10.0 * (f * f * f) / (exp((19e3 * f / temperature)) - 1.0);
    }

    return O;
}

// Stars
fn stars(coord : vec2<f32>, colorshift_t : f32, noise_t : f32) -> vec3<f32> {
    var luminance : f32 = max(0.0, (hash12(coord) - 0.985));
    var temperature : f32 = (hash12(coord + uni.iResolution.xy) * 6000.0) + 4000.0;
    var colorshift : vec3<f32> = normalize(colorednoise(coord, colorshift_t));
    return (luminance * noise2(coord, noise_t)) * blackbody(temperature) * 4.0 * (colorshift * 0.5 + 1.0);
}

// Galaxy
fn galaxygas(coord : vec2<f32>, n : f32) -> f32 {
    return max(0.0, fbm2((coord * 4.0 * n) + fbm2(coord * 4.0 * n)) - 0.35);
}

fn galaxydust(coord : vec2<f32>, n : f32) -> f32 {
    return max(0.0, fbm2((coord * 2.0 * n) + fbm2(coord * 2.0 * n) + vec2<f32>(4.0, 4.0)) - 0.5);
}

// Nebula
fn nebula(coord : vec2<f32>, n : f32) -> f32 {
    var gas0 : f32 = max(0.0, fbm2((coord * 2.0 * n) + fbm2(coord * 2.0 * n) + vec2<f32>(4.0, 4.0)) - length(coord));
    var gas1 : f32 = max(0.0, fbm2((coord * 4.0 * n) + fbm2(coord * 2.0 * n) + vec2<f32>(4.0, 4.0)) - length(coord * 1.01));
    return max(0.0, gas0 - gas1);
}

fn toLinear(sRGB: vec4<f32>) -> vec4<f32> {
    let cutoff = vec4<f32>(sRGB < vec4<f32>(0.04045));
    let higher = pow((sRGB + vec4<f32>(0.055)) / vec4<f32>(1.055), vec4<f32>(2.4));
    let lower = sRGB / vec4<f32>(12.92);

    return mix(higher, lower, cutoff);
}

@compute @workgroup_size(8, 8, 1)
fn update(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let R: vec2<f32> = uni.iResolution.xy;
    let y_inverted_location = vec2<i32>(i32(invocation_id.x), i32(R.y) - i32(invocation_id.y));
    let location = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));
	var fragCoord = vec2<f32>(f32(invocation_id.x), f32(invocation_id.y) );

    var uv : vec2<f32> = (fragCoord - 0.5 * uni.iResolution.xy) / max(uni.iResolution.x, uni.iResolution.y) * 2.;

    if (fbm1((uv.x + 4.0) * 4.0) > (uv.y + 0.5) * 4.0) {
        textureStore(texture, y_inverted_location, vec4<f32>(0.0, 0.0, 0.0, 1.0));
        return;
    }

    // var frame : i32 = i32(uni.iFrame);

    var t : f32 = 1.0;// + (uni.iTimeDelta * 0.3);
    var star : vec3<f32> = stars(fragCoord, uni.iTime * 3.0, uni.iTime * 0.7);
    var gas : f32 = galaxygas(uv, t);
    var dust : vec3<f32> = galaxydust(uv, t) * vec3<f32>(0.500, 0.400, 0.300);
    var nebulae : vec3<f32> = nebula(uv, t)  * vec3<f32>(0.600, 0.500, 0.750);
    var color : vec3<f32> = star + mix(vec3<f32>(gas), dust * 0.5, 0.75) + nebulae;
    // var color : vec3<f32> = mix(vec3<f32>(gas), dust * 0.5, 0.75) + nebulae;

	var fragColor = vec4<f32>(color, 1.0);
    textureStore(texture, y_inverted_location, toLinear(fragColor));
} 






// // Something like the Night Sky by Zi7ar21 --- February 1st, 2020
// // Updated February 1st, 17:15 Mountain Time

// // If you didn't find this on Shadertoy, the original can be found at:
// // https://www.shadertoy.com/view/ttcfRH

// //
// // This is a somewhat Night-Sky like scene. It is rendered in 2D, and is NOT physically accurate.
// // I didn't take into account proper gas colors, gas distribution, etc.
// // Also the stars have a Blackbody pallete but the probability of a random sampled star's
// // color and luminance is not based on something accurate like Hertzsprung-Russell.
// //

// // Hashes https://www.shadertoy.com/view/4djSRW

// fn hash11(p_in : f32) -> f32 {
//     var p : f32 = fract(p_in * 0.1031);
//     p *= p + 33.33;
//     p *= p + p;
//     return fract(p);
// }

// fn hash12(pos : vec2<f32>) -> f32 {
// 	var p3 : vec3<f32> = fract(vec3<f32>(pos.xyx) * 0.1031);
//     var p33 : vec3<f32> = p3 + vec3<f32>(dot(p3, p3.yzx + 33.33));
//     return fract((p33.x + p33.y) * p33.z);
// }

// fn hash13(p3_in : vec3<f32>) -> f32 {
// 	var p3 : vec3<f32> = fract(p3_in * 0.1031);
//     p3 += vec3<f32>(dot(p3, p3.zyx + 31.32));
//     return fract((p3.x + p3.y) * p3.z);
// }

// fn hash33(p3_in : vec3<f32>) -> vec3<f32> {
// 	var p3 : vec3<f32> = fract(p3_in * vec3<f32>(0.1031, 0.1030, 0.0973));
//     p3 += vec3<f32>(dot(p3, p3.yxz + 33.33));
//     return fract((p3.xxy + p3.yxx) * p3.zyx);
// }

// fn noise1(n : vec2<f32>) -> f32 {
//     var b : vec4<f32> = vec4<f32>(floor(n), ceil(n)); 
//     var f : vec2<f32> = smoothstep(vec2<f32>(0.0), vec2<f32>(1.0), fract(n));
// 	return mix(mix(hash12(b.xy), hash12(b.zy), f.x), mix(hash12(b.xw), hash12(b.zw), f.x), f.y);
// }

// fn modulo(x : f32, y : f32) -> f32 {
//     return x - y * floor(x/y);
// }

// fn octaves() -> i32 {
//     return 8;
// }

// // Atmospheric Distortion "Twinkling"
// fn noise2(coord : vec2<f32>, n : f32) -> f32 {
//     var componenta : f32 = hash13(vec3(coord, round(n - 0.5)));
//     var componentb : f32 = hash13(vec3(coord, round(n + 0.5)));
//     var componentc : f32 = mix(componenta, componentb, modulo(n, 1.0));
//     return componentc;
// }

// // FBM Terrain Line
// fn noise3(coord : f32) -> f32 {
//     var componenta : f32 = hash11(round(coord - 0.5));
//     var componentb : f32 = hash11(round(coord + 0.5));
//     return mix(componenta, componentb, modulo(coord, 1.0));
// }

// // Color Offset, as Reccomended by user "elenzil"
// // https://www.shadertoy.com/user/elenzil
// fn colorednoise(coord : vec2<f32>, n : f32) -> vec3<f32> {
//     var componenta : vec3<f32> = hash33(vec3<f32>(coord, round(n - 0.5)));
//     var componentb : vec3<f32> = hash33(vec3<f32>(coord, round(n + 0.5)));
//     var componentc : vec3<f32> = mix(componenta, componentb, modulo(n, 1.0));
//     return componentc;
// }

// // FBM https://www.shadertoy.com/view/3dSBRh
// // #define octaves 8
// fn fbm2(x_in : vec2<f32>) -> f32 {
// 	var v : f32 = 0.0;
// 	var a : f32 = 0.4;
//     var x : vec2<f32> = x_in;
// 	for(var i : i32 = 0; i < octaves(); i++) {
// 		v += a * noise1(x);
// 		x  = x * 2.0;
// 		a *= 0.6;
// 	}

// 	return v;
// }

// fn fbm1(x_in : f32) -> f32 {
// 	var v : f32 = 0.0;
// 	var a : f32 = 0.5;
//     var x : f32 = x_in;
// 	for(var i : i32 = 0; i < octaves(); i++) {
// 		v += a * noise3(x);
// 		x  = x * 2.0;
// 		a *= 0.5;
// 	}

// 	return v;
// }

// // Blackbody Coloration (Made into a Function by LoicVDB)
// // https://www.shadertoy.com/view/4tdGWM
// fn blackbody(temperature : f32) -> vec3<f32> {
//     var O : vec3<f32> = vec3<f32>(0.0);

//     for(var i : f32 = 0.0; i < 3.0; i += 0.1) {
//         var f : f32 = 1.0 + 0.5 * i;

//         O[i32(i)] += 10.0 * (f * f * f) / (exp((19e3 * f / temperature)) - 1.0);
//     }

//     return O;
// }

// // Stars
// fn stars(coord : vec2<f32>) -> vec3<f32> {
//     var luminance : f32 = max(0.0, (hash12(coord) - 0.985));
//     var temperature : f32 = (hash12(coord + uni.iResolution.xy) * 6000.0) + 4000.0;
//     var colorshift : vec3<f32> = normalize(colorednoise(coord, f32(uni.iTime * 16.0)));
//     return (luminance * noise2(coord, uni.iTime * 4.0)) * blackbody(temperature) * 4.0 * (colorshift * 0.5 + 1.0);
// }

// // Galaxy
// fn galaxygas(coord : vec2<f32>) -> f32 {
//     return max(0.0, fbm2((coord * 4.0) + fbm2(coord * 4.0)) - 0.35);
// }

// fn galaxydust(coord : vec2<f32>) -> f32 {
//     return max(0.0, fbm2((coord * 2.0) + fbm2(coord * 2.0) + vec2<f32>(4.0, 4.0)) - 0.5);
// }

// // Nebula
// fn nebula(coord : vec2<f32>) -> f32 {
//     var gas0 : f32 = max(0.0, fbm2((coord * 2.0) + fbm2(coord * 2.0) + vec2<f32>(4.0, 4.0)) - length(coord));
//     var gas1 : f32 = max(0.0, fbm2((coord * 4.0) + fbm2(coord * 2.0) + vec2<f32>(4.0, 4.0)) - length(coord * 1.01));
//     return max(0.0, gas0 - gas1);
// }

// fn toLinear(sRGB: vec4<f32>) -> vec4<f32> {
//     let cutoff = vec4<f32>(sRGB < vec4<f32>(0.04045));
//     let higher = pow((sRGB + vec4<f32>(0.055)) / vec4<f32>(1.055), vec4<f32>(2.4));
//     let lower = sRGB / vec4<f32>(12.92);

//     return mix(higher, lower, cutoff);
// }

// @compute @workgroup_size(8, 8, 1)
// fn update(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
//     let R: vec2<f32> = uni.iResolution.xy;
//     let y_inverted_location = vec2<i32>(i32(invocation_id.x), i32(R.y) - i32(invocation_id.y));
//     let location = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));
// 	var fragCoord = vec2<f32>(f32(invocation_id.x), f32(invocation_id.y) );

//     var uv : vec2<f32> = (fragCoord - 0.5 * uni.iResolution.xy) / max(uni.iResolution.x, uni.iResolution.y) * 2.;

//     if (fbm1((uv.x + 4.0) * 4.0) > (uv.y + 0.5) * 4.0) {
//         textureStore(texture, y_inverted_location, vec4<f32>(0.0, 0.0, 0.0, 1.0));
//         return;
//     }

//     var star : vec3<f32> = stars(fragCoord);
//     var gas : f32 = galaxygas(uv);
//     var dust : vec3<f32> = galaxydust(uv) * vec3<f32>(0.500, 0.400, 0.300);
//     var nebulae : vec3<f32> = nebula(uv)  * vec3<f32>(0.600, 0.500, 0.750);
//     var color : vec3<f32> = star + mix(vec3<f32>(gas), dust * 0.5, 0.75) + nebulae;
//     // var color : vec3<f32> = star + mix(vec3<f32>(gas), dust * 0.5, 0.75);

// 	var fragColor = vec4<f32>(color, 1.0);
//     textureStore(texture, y_inverted_location, toLinear(fragColor));
// } 



