using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using System.Numerics;

namespace Isbl.NetVR.Binary {

    public sealed class Controller: IEquatable<Controller>, ICloneable {
        public byte interaction_profile;
        public byte user_path;
        public Pose pose;

        public Controller(byte _interaction_profile, byte _user_path, Pose _pose) {
            interaction_profile = _interaction_profile;
            user_path = _user_path;
            if (_pose == null) throw new ArgumentNullException(nameof(_pose));
            pose = _pose;
        }

        public void Serialize(Serde.ISerializer serializer) {
            serializer.increase_container_depth();
            serializer.serialize_u8(interaction_profile);
            serializer.serialize_u8(user_path);
            pose.Serialize(serializer);
            serializer.decrease_container_depth();
        }

        public int BincodeSerialize(byte[] outputBuffer) => BincodeSerialize(new ArraySegment<byte>(outputBuffer));

        public int BincodeSerialize(ArraySegment<byte> outputBuffer) {
            Serde.ISerializer serializer = new Bincode.BincodeSerializer(outputBuffer);
            Serialize(serializer);
            return serializer.get_buffer_offset();
        }

        public byte[] BincodeSerialize()  {
            Serde.ISerializer serializer = new Bincode.BincodeSerializer();
            Serialize(serializer);
            return serializer.get_bytes();
        }

        public static Controller Deserialize(Serde.IDeserializer deserializer) {
            deserializer.increase_container_depth();
            Controller obj = new Controller(
            	deserializer.deserialize_u8(),
            	deserializer.deserialize_u8(),
            	Pose.Deserialize(deserializer));
            deserializer.decrease_container_depth();
            return obj;
        }

        public static Controller BincodeDeserialize(byte[] input) => BincodeDeserialize(new ArraySegment<byte>(input));

        public static Controller BincodeDeserialize(ArraySegment<byte> input) {
            if (input == null) {
                 throw new Serde.DeserializationException("Cannot deserialize null array");
            }
            Serde.IDeserializer deserializer = new Bincode.BincodeDeserializer(input);
            Controller value = Deserialize(deserializer);
            if (deserializer.get_buffer_offset() < input.Count) {
                 throw new Serde.DeserializationException("Some input bytes were not read");
            }
            return value;
        }
        public override bool Equals(object obj) => obj is Controller other && Equals(other);

        public static bool operator ==(Controller left, Controller right) => Equals(left, right);

        public static bool operator !=(Controller left, Controller right) => !Equals(left, right);

        public bool Equals(Controller other) {
            if (other == null) return false;
            if (ReferenceEquals(this, other)) return true;
            if (!interaction_profile.Equals(other.interaction_profile)) return false;
            if (!user_path.Equals(other.user_path)) return false;
            if (!pose.Equals(other.pose)) return false;
            return true;
        }

        public override int GetHashCode() {
            unchecked {
                int value = 7;
                value = 31 * value + interaction_profile.GetHashCode();
                value = 31 * value + user_path.GetHashCode();
                value = 31 * value + pose.GetHashCode();
                return value;
            }
        }

        /// <summary>Creates a shallow clone of the object.</summary>
        public Controller Clone() => (Controller)MemberwiseClone();

        object ICloneable.Clone() => Clone();

    }

} // end of namespace Isbl.NetVR.Binary
